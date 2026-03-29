use super::{Event, Message, TransportConfig};
use crate::BleStack;
use crate::external::{InitMessageService, RxMessageService, TxMessageService};
use arrayvec::ArrayVec;
use bt_hci::param::BdAddr;
use core::num::NonZeroU8;
use core::sync::atomic::Ordering;
use embassy_futures::join::join5;
use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_sync::rwlock::RwLock;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use generic_array::GenericArray;
use lokey::util::{debug, error, info, unwrap, warn};
use lokey::{Address, external, internal, storage};
use portable_atomic::{AtomicBool, AtomicU8};
use trouble_host::att::AttErrorCode;
use trouble_host::gap::{GapConfig, PeripheralConfig};
use trouble_host::gatt::{GattConnection, GattConnectionEvent, GattEvent};
use trouble_host::prelude::{
    AdStructure, Advertisement, AdvertisementParameters, AttributeServer, AttributeTable,
    BR_EDR_NOT_SUPPORTED, BluetoothUuid16, DefaultPacketPool, LE_GENERAL_DISCOVERABLE,
    RequestedConnParams, SecurityLevel,
};
use trouble_host::{BleHostError, BondInformation, Identity, IdentityResolvingKey, LongTermKey};

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
    min_connection_interval: Option<Duration>,
    max_connection_interval: Option<Duration>,
}

impl<Mcu, TxMessage, RxMessage, const CONN_MAX: usize> external::Transport
    for Transport<Mcu, TxMessage, RxMessage, CONN_MAX>
where
    Mcu: BleStack + 'static,
    TxMessage: crate::external::TxMessage,
    RxMessage: crate::external::RxMessage,
{
    type Config = TransportConfig;
    type Mcu = Mcu;
    type TxMessage = TxMessage;
    type RxMessage = RxMessage;

    async fn create<T>(
        config: Self::Config,
        mcu: &'static Self::Mcu,
        _: Address,
        internal_channel: &'static internal::Channel<T>,
    ) -> Self
    where
        T: internal::Transport<Mcu = Self::Mcu>,
    {
        Self {
            tx_channel: Channel::new(),
            rx_channel: Channel::new(),
            name: config.name,
            num_profiles: config.num_profiles,
            appearance: config.appearance,
            mcu,
            internal_channel: internal_channel.as_dyn_ref(),
            min_connection_interval: config.min_connection_interval,
            max_connection_interval: config.max_connection_interval,
        }
    }

    async fn run<Storage>(&self, storage: &'static Storage)
    where
        Storage: storage::Storage,
    {
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

        let adv_service_uuids_16_tx = TxMessage::service_uuids_16();
        let adv_service_uuids_128_tx = TxMessage::service_uuids_128();
        let adv_service_uuids_16_rx = RxMessage::service_uuids_16();
        let adv_service_uuids_128_rx = RxMessage::service_uuids_128();

        let mut ad_structure_vec = ArrayVec::<AdStructure, 6>::new();
        ad_structure_vec.extend([
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::CompleteLocalName(self.name.as_bytes()),
        ]);
        if !adv_service_uuids_16_tx.is_empty() {
            ad_structure_vec.push(AdStructure::ServiceUuids16(&adv_service_uuids_16_tx));
        }
        if !adv_service_uuids_16_rx.is_empty() {
            ad_structure_vec.push(AdStructure::ServiceUuids16(&adv_service_uuids_16_rx));
        }
        if !adv_service_uuids_128_tx.is_empty() {
            ad_structure_vec.push(AdStructure::ServiceUuids128(&adv_service_uuids_128_tx));
        }
        if !adv_service_uuids_128_rx.is_empty() {
            ad_structure_vec.push(AdStructure::ServiceUuids128(&adv_service_uuids_128_rx));
        }

        let adv_data_len = unwrap!(AdStructure::encode_slice(&ad_structure_vec, &mut adv_data));

        // TODO: add services to scan data
        let scan_data = [0; 31];

        let mut bond_infos = ArrayVec::<_, MAX_NUM_BOND_INFOS>::new();
        for i in 0..num_profiles.get() {
            match storage.fetch::<StoredBondInformation>(i).await {
                Ok(v) => bond_infos.push(v),
                Err(e) => {
                    #[cfg(feature = "defmt")]
                    let e = defmt::Debug2Format(&e);
                    error!("Failed to read bond info from flash: {}", e);
                    bond_infos.push(None);
                }
            }
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

                let scannable = match &bond_infos.lock().await[profile_index as usize] {
                    Some(bond_info) => {
                        #[cfg(feature = "defmt")]
                        debug!("Adding existing bond info: {}", bond_info);
                        #[cfg(not(feature = "defmt"))]
                        debug!("Adding existing bond info: {:?}", bond_info);
                        if let Err(e) = ble_stack.add_bond_information(bond_info.0.clone()) {
                            error!("Failed to add bond info: {}", e);
                        }
                        false
                    }
                    None => {
                        debug!("No existing bond info found");
                        true
                    }
                };

                let adv = Advertisement::ConnectableScannableUndirected {
                    adv_data: &adv_data[..adv_data_len],
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

                if self.min_connection_interval.is_some() || self.max_connection_interval.is_some()
                {
                    debug!("Updating connection parameters");
                    let mut conn_params = RequestedConnParams::default();
                    if let Some(v) = self.min_connection_interval {
                        conn_params.min_connection_interval = v;
                    }
                    if let Some(v) = self.max_connection_interval {
                        conn_params.max_connection_interval = v;
                    }
                    let result = new_connection
                        .update_connection_params(ble_stack, &conn_params)
                        .await;
                    if result.is_err() {
                        error!("Failed to update connection parameters");
                    }
                }

                let new_connection = match new_connection.with_attribute_server(&server) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Failed to add attribute server: {}", e);
                        continue;
                    }
                };
                if let Err(e) = new_connection.raw().set_bondable(true) {
                    error!("Failed to set connection as bondable: {}", e);
                }
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
                        GattConnectionEvent::PairingComplete {
                            security_level,
                            bond,
                        } => {
                            debug!(
                                "Received PairingComplete event with security level {}",
                                security_level
                            );
                            let Some(bond) = bond else {
                                debug!("No bond information received");
                                continue;
                            };
                            let store_new_bond_info = match &bond_infos.lock().await
                                [profile_index as usize]
                            {
                                Some(stored_bond_info) => {
                                    if stored_bond_info.0.ltk != bond.ltk {
                                        warn!(
                                            "LTK of new bond does not match LTK of stored bond, disconnecting..."
                                        );
                                        connection.raw().disconnect();
                                        self.internal_channel
                                            .send(Event::Disconnected { device_address })
                                            .await;
                                        break;
                                    } else {
                                        // Store new bond info if the address changed
                                        stored_bond_info.0.identity.bd_addr != bond.identity.bd_addr
                                    }
                                }
                                None => true,
                            };
                            if store_new_bond_info {
                                debug!("Writing bond info to flash");
                                if let Err(e) = storage
                                    .store(profile_index, StoredBondInformation::from_ref(&bond))
                                    .await
                                {
                                    #[cfg(feature = "defmt")]
                                    let e = defmt::Debug2Format(&e);
                                    error!("Failed to write bond info to flash: {}", e);
                                }
                                debug!("Adding bond info to stack");
                                if let Err(e) = ble_stack.add_bond_information(bond.clone()) {
                                    error!("Failed to add bond info to stack: {}", e);
                                }
                                bond_infos.lock().await[profile_index as usize] =
                                    Some(StoredBondInformation(bond));
                            }
                        }
                        GattConnectionEvent::PairingFailed(error) => {
                            error!("Pairing failed: {}", error);
                        }
                        GattConnectionEvent::PassKeyDisplay(pass_key) => {
                            debug!("Received PassKeyDisplay event: {}", pass_key.value());
                        }
                        GattConnectionEvent::PassKeyConfirm(pass_key) => {
                            debug!("Received PassKeyConfirm event: {}", pass_key.value());
                        }
                        GattConnectionEvent::PassKeyInput => {
                            debug!("Received PassKeyInput event");
                        }
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
                                        rx_message_service.receive(write_event, connection).await
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
                        GattConnectionEvent::RequestConnectionParams(v) => {
                            debug!("Received RequestConnectionParams event: {:?}", v);
                        }
                        GattConnectionEvent::DataLengthUpdated { .. } => {
                            debug!("Received DataLengthUpdated event");
                        }
                        GattConnectionEvent::PhyUpdated { .. } => {
                            debug!("Received PhyUpdated event");
                        }
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
            let mut receiver = unwrap!(self.internal_channel.receiver::<Message>());
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
                            if let Err(e) =
                                storage.remove::<StoredBondInformation>(profile_index).await
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
                        if let Err(e) = storage.remove::<StoredBondInformation>(profile_index).await
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
                            if let Err(e) = storage.remove::<StoredBondInformation>(i).await {
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(transparent)]
struct StoredBondInformation(BondInformation);

impl StoredBondInformation {
    fn from_ref(bond_info: &BondInformation) -> &Self {
        // SAFETY: StoredBondInformation is #[repr(transparent)] and has the same layout as
        // BondInformation
        unsafe { core::mem::transmute::<&BondInformation, &StoredBondInformation>(bond_info) }
    }
}

impl storage::Entry for StoredBondInformation {
    type Size = typenum::U40;
    type TagParams = u8;

    fn tag(params: Self::TagParams) -> [u8; 8] {
        [0x68, 0xb6, 0xa9, 0x22, 0xdc, 0xd9, 0xef, params]
    }

    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self>
    where
        Self: Sized,
    {
        let bytes = bytes.into_array::<40>();

        let has_irk = bytes[0] & 0b01 != 0;
        let is_bonded = bytes[0] & 0b10 != 0;

        let ltk = LongTermKey::from_le_bytes(bytes[1..17].try_into().unwrap());

        let address = BdAddr::new(bytes[17..23].try_into().unwrap());
        let irk =
            has_irk.then(|| IdentityResolvingKey::from_le_bytes(bytes[23..39].try_into().unwrap()));
        let identity = Identity {
            bd_addr: address,
            irk,
        };

        let security_level = match bytes[39] {
            0 => SecurityLevel::NoEncryption,
            1 => SecurityLevel::Encrypted,
            2 => SecurityLevel::EncryptedAuthenticated,
            v => {
                error!("Unknown security level in bond info: {}", v);
                return None;
            }
        };

        Some(StoredBondInformation(BondInformation::new(
            identity,
            ltk,
            security_level,
            is_bonded,
        )))
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        let mut bytes = [0; 40];

        bytes[0] = self.0.identity.irk.is_some() as u8 | (self.0.is_bonded as u8) << 1;

        bytes[1..17].copy_from_slice(&self.0.ltk.to_le_bytes());

        bytes[17..23].copy_from_slice(&self.0.identity.bd_addr.into_inner());
        if let Some(irk) = &self.0.identity.irk {
            bytes[23..39].copy_from_slice(&irk.to_le_bytes());
        }

        bytes[39] = match self.0.security_level {
            SecurityLevel::NoEncryption => 0,
            SecurityLevel::Encrypted => 1,
            SecurityLevel::EncryptedAuthenticated => 2,
        };

        bytes.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bond_info_serialization1() {
        let bond_info = BondInformation::new(
            Identity {
                bd_addr: BdAddr::new([1, 2, 3, 4, 5, 6]),
                irk: Some(IdentityResolvingKey::from_le_bytes([7; 16])),
            },
            LongTermKey::from_le_bytes([8; 16]),
            SecurityLevel::EncryptedAuthenticated,
            true,
        );
        let stored = StoredBondInformation(bond_info.clone());
        let bytes = storage::Entry::to_bytes(&stored);
        let deserialized = <StoredBondInformation as storage::Entry>::from_bytes(&bytes).unwrap();
        assert_eq!(stored.0.identity.bd_addr, deserialized.0.identity.bd_addr);
        assert_eq!(stored.0.identity.irk, deserialized.0.identity.irk);
        assert_eq!(stored.0.ltk, deserialized.0.ltk);
        assert_eq!(stored.0.security_level, deserialized.0.security_level);
        assert_eq!(stored.0.is_bonded, deserialized.0.is_bonded);
    }

    #[test]
    fn bond_info_serialization2() {
        let bond_info = BondInformation::new(
            Identity {
                bd_addr: BdAddr::new([21, 22, 23, 24, 25, 26]),
                irk: None,
            },
            LongTermKey::from_le_bytes([42; 16]),
            SecurityLevel::NoEncryption,
            false,
        );
        let stored = StoredBondInformation(bond_info.clone());
        let bytes = storage::Entry::to_bytes(&stored);
        let deserialized = <StoredBondInformation as storage::Entry>::from_bytes(&bytes).unwrap();
        assert_eq!(stored.0.identity.bd_addr, deserialized.0.identity.bd_addr);
        assert_eq!(stored.0.identity.irk, deserialized.0.identity.irk);
        assert_eq!(stored.0.ltk, deserialized.0.ltk);
        assert_eq!(stored.0.security_level, deserialized.0.security_level);
        assert_eq!(stored.0.is_bonded, deserialized.0.is_bonded);
    }
}
