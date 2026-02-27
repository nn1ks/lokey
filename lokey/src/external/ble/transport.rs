use super::{Event, Message, TransportConfig};
use crate::external::ble::{self, InitMessageService, RxMessageService, TxMessageService};
use crate::mcu::{self, McuBle, McuStorage, storage};
use crate::util::{debug, error, info, unwrap, warn};
use crate::{Address, external, internal};
use arrayvec::ArrayVec;
use core::num::NonZeroU8;
use core::sync::atomic::Ordering;
use embassy_futures::join::join5;
use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_sync::rwlock::RwLock;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use generic_array::GenericArray;
use portable_atomic::{AtomicBool, AtomicU8};
use trouble_host::att::AttErrorCode;
use trouble_host::gap::{GapConfig, PeripheralConfig};
use trouble_host::gatt::{GattConnection, GattConnectionEvent, GattEvent};
use trouble_host::prelude::{
    AdStructure, Advertisement, AdvertisementParameters, AttributeServer, AttributeTable,
    BR_EDR_NOT_SUPPORTED, BluetoothUuid16, DefaultPacketPool, LE_GENERAL_DISCOVERABLE,
};
use trouble_host::{BleHostError, BondInformation};

// TODO: Don't hardcode maximum number of bond infos
const MAX_NUM_BOND_INFOS: usize = 10;

static ACTIVE_SIGNAL: Signal<CriticalSectionRawMutex, bool> = Signal::new();
static IS_ACTIVE: AtomicBool = AtomicBool::new(true);

pub struct Transport<Mcu: 'static, TxMessages, RxMessages, const CONN_MAX: usize = 1> {
    tx_channel: Channel<CriticalSectionRawMutex, TxMessages, 1>,
    rx_channel: Channel<CriticalSectionRawMutex, RxMessages, 1>,
    name: &'static str,
    num_profiles: u8,
    appearance: &'static BluetoothUuid16,
    mcu: &'static Mcu,
    internal_channel: internal::DynChannelRef<'static>,
}

impl<Mcu, TxMessage, RxMessage, const CONN_MAX: usize> external::Transport
    for Transport<Mcu, TxMessage, RxMessage, CONN_MAX>
where
    Mcu: mcu::Mcu + McuBle + McuStorage,
    TxMessage: ble::TxMessage,
    RxMessage: ble::RxMessage,
{
    type Config = TransportConfig;
    type Mcu = Mcu;
    type TxMessage = TxMessage;
    type RxMessage = RxMessage;

    async fn create<T: internal::Transport<Mcu = Self::Mcu>>(
        config: Self::Config,
        mcu: &'static Self::Mcu,
        _: Address,
        internal_channel: &'static internal::Channel<T>,
    ) -> Self {
        Self {
            tx_channel: Channel::new(),
            rx_channel: Channel::new(),
            name: config.name,
            num_profiles: config.num_profiles,
            appearance: config.appearance,
            mcu,
            internal_channel: internal_channel.as_dyn_ref(),
        }
    }

    async fn run(&self) {
        // TODO: use TxMessage::ATTRIBUTE_COUNT and TxMessage::CCCD_MAX
        const ATT_MAX: usize = 50;
        const CCCD_MAX: usize = 50;

        let mut table = AttributeTable::<'_, NoopRawMutex, ATT_MAX>::new();

        let gap_config = GapConfig::Peripheral(PeripheralConfig {
            name: self.name,
            appearance: self.appearance,
        });
        if let Err(e) = gap_config.build(&mut table) {
            error!("Failed to set GAP config for BLE transport: {}", e);
        }

        let tx_message_service = TxMessage::MessageService::init(&mut table);
        let rx_message_service = RxMessage::MessageService::init(&mut table);

        let server = AttributeServer::<
            '_,
            NoopRawMutex,
            DefaultPacketPool,
            ATT_MAX,
            CCCD_MAX,
            CONN_MAX,
        >::new(table);

        let Some(num_profiles) = NonZeroU8::new(self.num_profiles) else {
            return;
        };

        let ble_stack = self.mcu.ble_stack();
        let mut host = ble_stack.build();

        let run = async {
            loop {
                if let Err(e) = host.runner.run().await {
                    #[cfg(feature = "defmt")]
                    let e = defmt::Debug2Format(&e);
                    error!("BLE run error: {}", e);
                    Timer::after_secs(1).await;
                }
            }
        };

        let adv_params = AdvertisementParameters::default();
        let mut adv_data = [0; 31];
        // const APPEARANCE_ADV_TYPE: u8 = 0x19;
        // const KEYBOARD_APPEARANCE: &[u8] = &[0xC1, 0x03];

        let adv_service_uuids_16_tx = TxMessage::service_uuids_16();
        let adv_service_uuids_128_tx = TxMessage::service_uuids_128();
        let adv_service_uuids_16_rx = RxMessage::service_uuids_16();
        let adv_service_uuids_128_rx = RxMessage::service_uuids_128();

        unwrap!(AdStructure::encode_slice(
            &[
                AdStructure::CompleteLocalName(self.name.as_bytes()),
                AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
                AdStructure::ServiceUuids16(&adv_service_uuids_16_tx),
                AdStructure::ServiceUuids16(&adv_service_uuids_16_rx),
                AdStructure::ServiceUuids128(&adv_service_uuids_128_tx),
                AdStructure::ServiceUuids128(&adv_service_uuids_128_rx),
                // AdStructure::Unknown {
                //     ty: APPEARANCE_ADV_TYPE,
                //     data: KEYBOARD_APPEARANCE,
                // },
            ],
            &mut adv_data,
        ));

        // TODO: add services to scan data
        let scan_data = [0; 31];

        let mut bond_infos = ArrayVec::<_, MAX_NUM_BOND_INFOS>::new();
        for _ in 0..num_profiles.get() {
            // match mcu.storage().fetch::<BondInformation>(i).await {
            //     Ok(v) => bond_infos.push(v),
            //     Err(e) => {
            //         #[cfg(feature = "defmt")]
            //         let e = defmt::Debug2Format(&e);
            //         error!("Failed to read bond info from flash: {}", e);
            //         bond_infos.push(None);
            //     }
            // }
            bond_infos.push(None::<BondInformation>);
        }
        #[cfg(feature = "defmt")]
        info!("Stored bond infos: {}", defmt::Debug2Format(&bond_infos));
        #[cfg(not(feature = "defmt"))]
        info!("Stored bond infos: {:?}", bond_infos);
        let bond_infos = Mutex::<CriticalSectionRawMutex, _>::new(bond_infos);

        let connection = RwLock::<
            CriticalSectionRawMutex,
            Option<GattConnection<'_, '_, DefaultPacketPool>>,
        >::new(None);

        let cancel_activation_wait = Signal::<CriticalSectionRawMutex, ()>::new();
        let cancel_advertisement = Signal::<CriticalSectionRawMutex, ()>::new();
        let active_profile_index: AtomicU8 = AtomicU8::new(0);
        let advertise = async {
            loop {
                *connection.write().await = None;
                cancel_advertisement.reset();

                while !IS_ACTIVE.load(Ordering::Acquire) {
                    cancel_activation_wait.wait().await;
                }

                let profile_index = active_profile_index.load(Ordering::SeqCst);

                // for bond_info in ble_stack.get_bond_information() {
                //     warn!("BOND_INFO: Some({})", bond_info);
                //     if let Err(e) = ble_stack.remove_bond_information(bond_info.address) {
                //         error!("Failed to remove bond info: {}", e);
                //     }
                // }
                let scannable = match &bond_infos.lock().await[profile_index as usize] {
                    Some(bond_info) => {
                        debug!("Adding existing bond info: {}", bond_info);
                        // if let Err(e) = ble_stack.add_bond_information(bond_info.clone()) {
                        //     error!("Failed to add bond info: {}", e);
                        // }
                        false
                    }
                    None => {
                        debug!("No existing bond info found");
                        true
                    }
                };

                let adv = Advertisement::ConnectableScannableUndirected {
                    adv_data: &adv_data,
                    scan_data: &scan_data,
                };

                info!("Starting BLE advertisement");
                self.internal_channel
                    .send(Event::StartedAdvertising { scannable })
                    .await;

                let advertiser = match select(
                    host.peripheral.advertise(&adv_params, adv),
                    cancel_advertisement.wait(),
                )
                .await
                {
                    Either::First(Ok(v)) => v,
                    Either::First(Err(e)) => {
                        match e {
                            BleHostError::Controller(_) => {
                                error!("Failed to advertise: Controller error")
                            }
                            BleHostError::BleHost(e) => {
                                error!("Failed to advertise: {}", e)
                            }
                        }
                        self.internal_channel
                            .send(Event::StoppedAdvertising { scannable })
                            .await;
                        continue;
                    }
                    Either::Second(()) => {
                        debug!("Cancelling advertisement");
                        self.internal_channel
                            .send(Event::StoppedAdvertising { scannable })
                            .await;
                        continue;
                    }
                };

                let new_connection =
                    match select(advertiser.accept(), cancel_advertisement.wait()).await {
                        Either::First(Ok(v)) => v,
                        Either::First(Err(e)) => {
                            error!("Failed to accept connection: {}", e);
                            self.internal_channel
                                .send(Event::StoppedAdvertising { scannable })
                                .await;
                            continue;
                        }
                        Either::Second(()) => {
                            debug!("Cancelling advertisement");
                            self.internal_channel
                                .send(Event::StoppedAdvertising { scannable })
                                .await;
                            continue;
                        }
                    };
                self.internal_channel
                    .send(Event::StoppedAdvertising { scannable })
                    .await;
                let device_address = Address(new_connection.peer_address().into_inner());
                let new_connection = match new_connection.with_attribute_server(&server) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Failed to add attribute server: {}", e);
                        continue;
                    }
                };
                *connection.write().await = Some(new_connection);

                info!("BLE connected");
                self.internal_channel
                    .send(Event::Connected { device_address })
                    .await;

                loop {
                    let connection = connection.read().await;
                    let connection = connection.as_ref().unwrap();
                    if !connection.raw().is_connected() {
                        debug!("BLE is not connected");
                        self.internal_channel
                            .send(Event::Disconnected { device_address })
                            .await;
                        break;
                    }
                    match connection.next().await {
                        GattConnectionEvent::Disconnected { reason } => {
                            info!("BLE disconnected (reason: {})", reason);
                            self.internal_channel
                                .send(Event::Disconnected { device_address })
                                .await;
                            break;
                        }
                        // TODO
                        // GattConnectionEvent::Bonded { bond_info } => {
                        //     debug!("Received Bonded event");
                        //     let store_new_bond_info = match &bond_infos.lock().await
                        //         [profile_index as usize]
                        //     {
                        //         Some(stored_bond_info) => {
                        //             if stored_bond_info.ltk != bond_info.ltk {
                        //                 warn!(
                        //                     "LTK of new bond does not match LTK of stored bond, disconnecting..."
                        //                 );
                        //                 connection.raw().disconnect();
                        //                 self.internal_channel
                        //                     .send(Event::Disconnected { device_address });
                        //                 break;
                        //             } else {
                        //                 // Store new bond info if the address changed
                        //                 stored_bond_info.address != bond_info.address
                        //             }
                        //         }
                        //         None => true,
                        //     };
                        //     if store_new_bond_info {
                        //         // debug!("Writing bond info to flash");
                        //         // if let Err(e) = mcu.storage().store(profile_index, &bond_info).await {
                        //         //     #[cfg(feature = "defmt")]
                        //         //     let e = defmt::Debug2Format(&e);
                        //         //     error!("Failed to write bond info to flash: {}", e);
                        //         // }
                        //         debug!("Adding bond info to stack");
                        //         if let Err(e) = ble_stack.add_bond_information(bond_info.clone()) {
                        //             error!("Failed to add bond info to stack: {}", e);
                        //         }
                        //         bond_infos.lock().await[profile_index as usize] = Some(bond_info);
                        //     }
                        // }
                        GattConnectionEvent::Gatt { event } => {
                            debug!("Received GATT event");
                            let result = if connection
                                .raw()
                                .security_level()
                                .map(|v| v.encrypted())
                                .unwrap_or(false)
                            {
                                if let GattEvent::Write(write_event) = &event
                                    && let Some(message) =
                                        rx_message_service.receive(write_event).await
                                {
                                    self.rx_channel.send(message).await;
                                }
                                event.accept()
                            } else {
                                warn!("Rejecting event because connection is not encrypted");
                                event.reject(AttErrorCode::INSUFFICIENT_ENCRYPTION)
                            };
                            match result {
                                Ok(reply) => reply.send().await,
                                Err(error) => error!("Failed to handle event: {}", error),
                            }
                        }
                        GattConnectionEvent::ConnectionParamsUpdated { .. } => {
                            debug!("Received ConnectionParamsUpdated event");
                        }
                        GattConnectionEvent::PhyUpdated { .. } => {
                            debug!("Received PhyUpdated event");
                        }
                        // TODO: remove
                        _ => {}
                    }
                }
                self.internal_channel
                    .send(Event::Disconnected { device_address })
                    .await;
            }
        };

        let handle_messages = async {
            loop {
                let message = self.tx_channel.receive().await;
                match &*connection.read().await {
                    Some(connection) => {
                        tx_message_service.send(message, connection).await;
                    }
                    None => info!("Ignoring external message because BLE is disconnected"),
                }
            }
        };

        let handle_internal_messages = async {
            let mut receiver = self.internal_channel.receiver::<Message>();
            loop {
                let message = receiver.next().await;
                match message {
                    Message::SelectProfile { index } => {
                        if index >= num_profiles.get() {
                            error!(
                                "Cannot select profile {} as number of profiles is set to {}",
                                index, num_profiles
                            );
                        } else {
                            info!("Switching to profile {}", index);
                            let is_different_profile =
                                active_profile_index.load(Ordering::SeqCst) != index;
                            if is_different_profile {
                                active_profile_index.store(index, Ordering::SeqCst);
                                if let Some(connection) = &*connection.read().await {
                                    connection.raw().disconnect();
                                }
                                cancel_advertisement.signal(());
                            }
                            self.internal_channel
                                .send(Event::SwitchedProfile {
                                    profile_index: index,
                                    changed: is_different_profile,
                                })
                                .await;
                        }
                    }
                    Message::SelectNextProfile => {
                        let active = active_profile_index.load(Ordering::SeqCst);
                        let new_profile_index = if active == num_profiles.get() - 1 {
                            0
                        } else {
                            active + 1
                        };
                        info!("Switching to profile {}", new_profile_index);
                        active_profile_index.store(new_profile_index, Ordering::SeqCst);
                        if let Some(connection) = &*connection.read().await {
                            connection.raw().disconnect();
                        }
                        cancel_advertisement.signal(());
                        self.internal_channel
                            .send(Event::SwitchedProfile {
                                profile_index: new_profile_index,
                                changed: num_profiles.get() > 1,
                            })
                            .await;
                    }
                    Message::SelectPreviousProfile => {
                        let active = active_profile_index.load(Ordering::SeqCst);
                        let new_profile_index = if active == 0 {
                            num_profiles.get() - 1
                        } else {
                            active - 1
                        };
                        info!("Switching to profile {}", new_profile_index);
                        active_profile_index.store(new_profile_index, Ordering::SeqCst);
                        if let Some(connection) = &*connection.read().await {
                            connection.raw().disconnect();
                        }
                        cancel_advertisement.signal(());
                        self.internal_channel
                            .send(Event::SwitchedProfile {
                                profile_index: new_profile_index,
                                changed: num_profiles.get() > 1,
                            })
                            .await;
                    }
                    Message::DisconnectActive => {
                        if let Some(connection) = &*connection.read().await {
                            connection.raw().disconnect();
                        }
                    }
                    Message::Clear { profile_index } => {
                        if profile_index >= num_profiles.get() {
                            error!(
                                "Cannot clear profile {} as number of profiles is set to {}",
                                profile_index, num_profiles
                            );
                        } else {
                            debug!("Removing bond info for profile {}", profile_index);
                            if let Err(e) = self
                                .mcu
                                .storage()
                                .remove::<BondInformation>(profile_index)
                                .await
                            {
                                #[cfg(feature = "defmt")]
                                let e = defmt::Debug2Format(&e);
                                error!("Failed to remove bond info: {}", e);
                            }
                            bond_infos.lock().await[profile_index as usize] = None;
                            if active_profile_index.load(Ordering::SeqCst) == profile_index {
                                if let Some(connection) = &*connection.read().await {
                                    connection.raw().disconnect();
                                }
                                cancel_advertisement.signal(());
                            }
                        }
                    }
                    Message::ClearActive => {
                        debug!("Removing bond info for active profile");
                        let profile_index = active_profile_index.load(Ordering::SeqCst);
                        if let Err(e) = self
                            .mcu
                            .storage()
                            .remove::<BondInformation>(profile_index)
                            .await
                        {
                            #[cfg(feature = "defmt")]
                            let e = defmt::Debug2Format(&e);
                            error!("Failed to remove bond info: {}", e);
                        }
                        bond_infos.lock().await[profile_index as usize] = None;
                        if let Some(connection) = &*connection.read().await {
                            connection.raw().disconnect();
                        }
                        cancel_advertisement.signal(());
                    }
                    Message::ClearAll => {
                        debug!("Removing all bond infos");
                        for i in 0..num_profiles.get() {
                            if let Err(e) = self.mcu.storage().remove::<BondInformation>(i).await {
                                #[cfg(feature = "defmt")]
                                let e = defmt::Debug2Format(&e);
                                error!("Failed to remove bond info: {}", e);
                            }
                            bond_infos.lock().await[i as usize] = None;
                        }
                        if let Some(connection) = &*connection.read().await {
                            connection.raw().disconnect();
                        }
                        cancel_advertisement.signal(());
                    }
                }
            }
        };

        let handle_activation = async {
            loop {
                ACTIVE_SIGNAL.wait().await;
                if let Some(connection) = &*connection.read().await {
                    connection.raw().disconnect();
                }
                cancel_activation_wait.signal(());
                cancel_advertisement.signal(());
            }
        };

        join5(
            run,
            advertise,
            handle_messages,
            handle_internal_messages,
            handle_activation,
        )
        .await;
    }

    async fn send(&self, message: Self::TxMessage) {
        self.tx_channel.send(message).await;
    }

    async fn receive(&self) -> Self::RxMessage {
        self.rx_channel.receive().await
    }

    async fn set_active(&self, value: bool) -> bool {
        info!(
            "Setting active status of external BLE transport to {}",
            value
        );
        IS_ACTIVE.store(value, Ordering::Release);
        ACTIVE_SIGNAL.signal(value);
        true
    }

    fn is_active(&self) -> bool {
        IS_ACTIVE.load(Ordering::Acquire)
    }
}

impl storage::Entry for BondInformation {
    type Size = typenum::U22;
    type TagParams = u8;

    fn tag(params: Self::TagParams) -> [u8; 8] {
        [0x68, 0xb6, 0xa9, 0x22, 0xdc, 0xd9, 0xef, params]
    }

    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self>
    where
        Self: Sized,
    {
        let _ = bytes;
        todo!()
        // let bytes = bytes.into_array::<22>();
        // let ltk = LongTermKey::from_le_bytes(bytes[..16].try_into().unwrap());
        // let address = BdAddr::new(bytes[16..].try_into().unwrap());
        // Some(BondInformation::new(address, ltk))
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        todo!()
        // let mut bytes = [0; 22];
        // bytes[..16].copy_from_slice(&self.ltk.to_le_bytes());
        // bytes[16..].copy_from_slice(&self.address.into_inner());
        // GenericArray::from_array(bytes)
    }
}
