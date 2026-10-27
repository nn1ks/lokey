use super::{Event, Message, TransportConfig};
use crate::BleStack;
use crate::external::bond_information::StoredBondInformation;
use crate::external::{InitMessageService, RxMessageService, TxMessageService, bond_information};
use arrayvec::ArrayVec;
use bt_hci::uuid::appearance;
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
use lokey::storage::WriteTransaction;
use lokey::util::{debug, error, info, unwrap, warn};
use lokey::{Address, external, internal, storage};
use portable_atomic::{AtomicBool, AtomicU8};
use trouble_host::gap::{GapConfig, PeripheralConfig};
use trouble_host::gatt::{GattConnection, GattConnectionEvent, GattEvent};
use trouble_host::prelude::{
    AdStructure, Advertisement, AdvertisementParameters, AttributeServer, AttributeTable,
    BR_EDR_NOT_SUPPORTED, BluetoothUuid16, DefaultPacketPool, LE_GENERAL_DISCOVERABLE,
    RequestedConnParams,
};
use trouble_host::{BleHostError, BondInformation};

// TODO: Don't hardcode maximum number of bond infos
pub(crate) const MAX_NUM_BOND_INFOS: usize = 10;

// TODO: use TxMessage::ATTRIBUTE_COUNT
const ATT_MAX: usize = 128;

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
    supervision_timeout: Option<Duration>,
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
            supervision_timeout: config.supervision_timeout,
        }
    }

    async fn run<Storage>(&self, storage: &'static Storage)
    where
        Storage: storage::Storage,
    {
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

        let server =
            AttributeServer::<'_, NoopRawMutex, DefaultPacketPool, ATT_MAX, CONN_MAX>::new(table);

        let Some(num_profiles) = NonZeroU8::new(self.num_profiles) else {
            warn!("BLE transport is configured with 0 profiles");
            return;
        };

        let profile_bond_infos = bond_information::read_all_from_storage(storage).await;

        let runner = Runner::new(
            storage,
            tx_message_service,
            rx_message_service,
            &server,
            profile_bond_infos,
            num_profiles,
        );

        join5(
            self.run_ble_host_runner(),
            self.advertise_and_handle_connections(&runner),
            self.handle_incoming_messages(&runner),
            self.handle_internal_messages(&runner),
            self.handle_activation(&runner),
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

struct Runner<'server, Storage: 'static, TxMessageService, RxMessageService, const CONN_MAX: usize>
{
    storage: &'static Storage,
    tx_message_service: TxMessageService,
    rx_message_service: RxMessageService,
    server: &'server AttributeServer<'static, NoopRawMutex, DefaultPacketPool, ATT_MAX, CONN_MAX>,
    connection: RwLock<
        CriticalSectionRawMutex,
        Option<GattConnection<'static, 'server, DefaultPacketPool>>,
    >,
    cancel_activation_wait: Signal<CriticalSectionRawMutex, ()>,
    cancel_advertisement: Signal<CriticalSectionRawMutex, ()>,
    active_profile_index: AtomicU8,
    profile_bond_infos:
        Mutex<CriticalSectionRawMutex, [Option<BondInformation>; MAX_NUM_BOND_INFOS]>,
    num_profiles: NonZeroU8,
}

impl<'server, Storage, TxMessageService, RxMessageService, const CONN_MAX: usize>
    Runner<'server, Storage, TxMessageService, RxMessageService, CONN_MAX>
where
    Storage: storage::Storage,
{
    fn new(
        storage: &'static Storage,
        tx_message_service: TxMessageService,
        rx_message_service: RxMessageService,
        server: &'server AttributeServer<
            'static,
            NoopRawMutex,
            DefaultPacketPool,
            ATT_MAX,
            CONN_MAX,
        >,
        profile_bond_infos: [Option<BondInformation>; MAX_NUM_BOND_INFOS],
        num_profiles: NonZeroU8,
    ) -> Self {
        Self {
            storage,
            tx_message_service,
            rx_message_service,
            server,
            connection: RwLock::new(None),
            cancel_activation_wait: Signal::new(),
            cancel_advertisement: Signal::new(),
            active_profile_index: AtomicU8::new(0),
            profile_bond_infos: Mutex::new(profile_bond_infos),
            num_profiles,
        }
    }

    async fn remove_bond(&self, profile_index: u8) {
        debug!("Removing bond info for profile {}", profile_index);
        {
            let mut write_transaction = self.storage.write_transaction().await;
            if let Err(e) = write_transaction
                .delete::<StoredBondInformation>(profile_index)
                .await
            {
                #[cfg(feature = "defmt")]
                let e = defmt::Debug2Format(&e);
                error!("Failed to remove bond info: {}", e);
            }
            if let Err(e) = write_transaction.commit().await {
                #[cfg(feature = "defmt")]
                let e = defmt::Debug2Format(&e);
                error!("Failed to commit to database: {}", e);
            }
        }
        self.profile_bond_infos.lock().await[profile_index as usize] = None;
    }

    async fn remove_all_bonds(&self) {
        debug!("Removing all bond infos");
        {
            let mut write_transaction = self.storage.write_transaction().await;
            for i in 0..self.num_profiles.get() {
                if let Err(e) = write_transaction.delete::<StoredBondInformation>(i).await {
                    #[cfg(feature = "defmt")]
                    let e = defmt::Debug2Format(&e);
                    error!("Failed to remove bond info: {}", e);
                }
                self.profile_bond_infos.lock().await[i as usize] = None;
            }
            if let Err(e) = write_transaction.commit().await {
                #[cfg(feature = "defmt")]
                let e = defmt::Debug2Format(&e);
                error!("Failed to commit to database: {}", e);
            }
        }
    }
}

enum ConnectionStatusResult {
    Connected,
    Disconnected,
}

impl<Mcu, TxMessage, RxMessage, const CONN_MAX: usize>
    Transport<Mcu, TxMessage, RxMessage, CONN_MAX>
where
    Mcu: BleStack + 'static,
    TxMessage: crate::external::TxMessage,
    RxMessage: crate::external::RxMessage,
{
    async fn handle_activation<'server, Storage>(
        &self,
        runner: &Runner<
            'server,
            Storage,
            TxMessage::MessageService,
            RxMessage::MessageService,
            CONN_MAX,
        >,
    ) where
        Storage: storage::Storage,
    {
        loop {
            let active = ACTIVE_SIGNAL.wait().await;
            info!("Active signal was set to: {}", active);
            if !active {
                if let Some(connection) = &*runner.connection.read().await {
                    connection.raw().disconnect();
                }
                runner.cancel_activation_wait.signal(());
                runner.cancel_advertisement.signal(());
            }
        }
    }

    async fn run_ble_host_runner(&self) {
        let ble_host_runner = self.mcu.ble_host_runner();
        loop {
            if let Err(e) = ble_host_runner.lock().await.run().await {
                #[cfg(feature = "defmt")]
                let e = defmt::Debug2Format(&e);
                error!("BLE run error: {}", e);
                Timer::after_secs(1).await;
            }
        }
    }

    async fn handle_incoming_messages<'server, Storage>(
        &self,
        runner: &Runner<
            'server,
            Storage,
            TxMessage::MessageService,
            RxMessage::MessageService,
            CONN_MAX,
        >,
    ) where
        Storage: storage::Storage,
    {
        loop {
            let message = self.tx_channel.receive().await;
            match &*runner.connection.read().await {
                Some(connection) => {
                    runner.tx_message_service.send(message, connection).await;
                }
                None => info!("Ignoring external message because BLE is disconnected"),
            }
        }
    }

    async fn handle_internal_messages<'server, Storage>(
        &self,
        runner: &Runner<
            'server,
            Storage,
            TxMessage::MessageService,
            RxMessage::MessageService,
            CONN_MAX,
        >,
    ) where
        Storage: storage::Storage,
    {
        let mut receiver = unwrap!(self.internal_channel.receiver::<Message>());
        loop {
            let message = receiver.next().await;
            match message {
                Message::SelectProfile { index } => {
                    if index >= runner.num_profiles.get() {
                        error!(
                            "Cannot select profile {} as number of profiles is set to {}",
                            index, runner.num_profiles
                        );
                    } else {
                        info!("Switching to profile {}", index);
                        let is_different_profile =
                            runner.active_profile_index.load(Ordering::SeqCst) != index;
                        if is_different_profile {
                            runner.active_profile_index.store(index, Ordering::SeqCst);
                            if let Some(connection) = &*runner.connection.read().await {
                                connection.raw().disconnect();
                            }
                            runner.cancel_advertisement.signal(());
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
                    let active = runner.active_profile_index.load(Ordering::SeqCst);
                    let new_profile_index = if active == runner.num_profiles.get() - 1 {
                        0
                    } else {
                        active + 1
                    };
                    info!("Switching to profile {}", new_profile_index);
                    runner
                        .active_profile_index
                        .store(new_profile_index, Ordering::SeqCst);
                    if let Some(connection) = &*runner.connection.read().await {
                        connection.raw().disconnect();
                    }
                    runner.cancel_advertisement.signal(());
                    self.internal_channel
                        .send(Event::SwitchedProfile {
                            profile_index: new_profile_index,
                            changed: runner.num_profiles.get() > 1,
                        })
                        .await;
                }
                Message::SelectPreviousProfile => {
                    let active = runner.active_profile_index.load(Ordering::SeqCst);
                    let new_profile_index = if active == 0 {
                        runner.num_profiles.get() - 1
                    } else {
                        active - 1
                    };
                    info!("Switching to profile {}", new_profile_index);
                    runner
                        .active_profile_index
                        .store(new_profile_index, Ordering::SeqCst);
                    if let Some(connection) = &*runner.connection.read().await {
                        connection.raw().disconnect();
                    }
                    runner.cancel_advertisement.signal(());
                    self.internal_channel
                        .send(Event::SwitchedProfile {
                            profile_index: new_profile_index,
                            changed: runner.num_profiles.get() > 1,
                        })
                        .await;
                }
                Message::DisconnectActive => {
                    if let Some(connection) = &*runner.connection.read().await {
                        connection.raw().disconnect();
                    }
                }
                Message::Clear { profile_index } => {
                    if profile_index >= runner.num_profiles.get() {
                        error!(
                            "Cannot clear profile {} as number of profiles is set to {}",
                            profile_index, runner.num_profiles
                        );
                    } else {
                        runner.remove_bond(profile_index).await;
                        if runner.active_profile_index.load(Ordering::SeqCst) == profile_index {
                            if let Some(connection) = &*runner.connection.read().await {
                                connection.raw().disconnect();
                            }
                            runner.cancel_advertisement.signal(());
                        }
                    }
                }
                Message::ClearActive => {
                    let profile_index = runner.active_profile_index.load(Ordering::SeqCst);
                    runner.remove_bond(profile_index).await;
                    if let Some(connection) = &*runner.connection.read().await {
                        connection.raw().disconnect();
                    }
                    runner.cancel_advertisement.signal(());
                }
                Message::ClearAll => {
                    runner.remove_all_bonds().await;
                    if let Some(connection) = &*runner.connection.read().await {
                        connection.raw().disconnect();
                    }
                    runner.cancel_advertisement.signal(());
                }
            }
        }
    }

    fn build_adv_data(&self, discoverable: bool) -> ArrayVec<u8, 31> {
        let mut adv_data = [0; 31];

        let appearance_bytes = self.appearance.to_be_bytes();
        let adv_service_uuids_16_tx = TxMessage::service_uuids_16();
        let adv_service_uuids_128_tx = TxMessage::service_uuids_128();
        let adv_service_uuids_16_rx = RxMessage::service_uuids_16();
        let adv_service_uuids_128_rx = RxMessage::service_uuids_128();

        let mut ad_structure_vec = ArrayVec::<AdStructure, 7>::new();
        let mut flags = BR_EDR_NOT_SUPPORTED;
        if discoverable {
            flags |= LE_GENERAL_DISCOVERABLE;
        }
        ad_structure_vec.extend([
            AdStructure::Flags(flags),
            AdStructure::CompleteLocalName(self.name.as_bytes()),
        ]);
        if self.appearance != &appearance::UNKNOWN {
            ad_structure_vec.push(AdStructure::Unknown {
                ty: 0x19, // Appearance
                data: &appearance_bytes,
            });
        }
        if !adv_service_uuids_16_tx.is_empty() {
            ad_structure_vec.push(AdStructure::IncompleteServiceUuids16(
                &adv_service_uuids_16_tx,
            ));
        }
        if !adv_service_uuids_16_rx.is_empty() {
            ad_structure_vec.push(AdStructure::IncompleteServiceUuids16(
                &adv_service_uuids_16_rx,
            ));
        }
        if !adv_service_uuids_128_tx.is_empty() {
            ad_structure_vec.push(AdStructure::IncompleteServiceUuids128(
                &adv_service_uuids_128_tx,
            ));
        }
        if !adv_service_uuids_128_rx.is_empty() {
            ad_structure_vec.push(AdStructure::IncompleteServiceUuids128(
                &adv_service_uuids_128_rx,
            ));
        }

        let adv_data_len = unwrap!(AdStructure::encode_slice(&ad_structure_vec, &mut adv_data));

        ArrayVec::try_from(&adv_data[..adv_data_len]).expect("adv data too large")
    }

    async fn advertise_and_handle_connections<'server, Storage>(
        &self,
        runner: &Runner<
            'server,
            Storage,
            TxMessage::MessageService,
            RxMessage::MessageService,
            CONN_MAX,
        >,
    ) where
        Storage: storage::Storage,
    {
        let ble_stack = self.mcu.ble_stack();
        let mut ble_host_peripheral = ble_stack.peripheral();

        let adv_params = AdvertisementParameters::default();
        let adv_data_discoverable = self.build_adv_data(true);
        let adv_data_non_discoverable = self.build_adv_data(false);

        // TODO: add services to scan data
        let scan_data = [0; 31];

        loop {
            *runner.connection.write().await = None;
            runner.cancel_advertisement.reset();

            while !IS_ACTIVE.load(Ordering::Acquire) {
                runner.cancel_activation_wait.wait().await;
            }

            let profile_index = runner.active_profile_index.load(Ordering::SeqCst);

            let discoverable = match &runner.profile_bond_infos.lock().await[profile_index as usize]
            {
                Some(bond_info) => {
                    #[cfg(feature = "defmt")]
                    debug!("Adding existing bond info: {}", bond_info);
                    #[cfg(not(feature = "defmt"))]
                    debug!("Adding existing bond info: {:?}", bond_info);
                    if let Err(e) = ble_stack.add_bond_information(bond_info.clone()) {
                        error!("Failed to add bond info: {}", e);
                    }
                    false
                }
                None => {
                    debug!("No existing bond info found");
                    true
                }
            };

            let adv_data = if discoverable {
                debug!("Advertising as discoverable");
                &adv_data_discoverable
            } else {
                debug!("Advertising as non-discoverable");
                &adv_data_non_discoverable
            };
            let adv = Advertisement::ConnectableScannableUndirected {
                adv_data,
                scan_data: &scan_data,
            };

            info!("Starting BLE advertisement");
            self.internal_channel
                .send(Event::StartedAdvertising { discoverable })
                .await;

            let advertiser = match select(
                ble_host_peripheral.advertise(&adv_params, adv),
                runner.cancel_advertisement.wait(),
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
                        .send(Event::StoppedAdvertising { discoverable })
                        .await;
                    Timer::after_millis(250).await;
                    continue;
                }
                Either::Second(()) => {
                    debug!("Cancelling advertisement");
                    self.internal_channel
                        .send(Event::StoppedAdvertising { discoverable })
                        .await;
                    continue;
                }
            };

            let new_connection =
                match select(advertiser.accept(), runner.cancel_advertisement.wait()).await {
                    Either::First(Ok(v)) => v,
                    Either::First(Err(e)) => {
                        error!("Failed to accept connection: {}", e);
                        self.internal_channel
                            .send(Event::StoppedAdvertising { discoverable })
                            .await;
                        Timer::after_millis(250).await;
                        continue;
                    }
                    Either::Second(()) => {
                        debug!("Cancelling advertisement");
                        self.internal_channel
                            .send(Event::StoppedAdvertising { discoverable })
                            .await;
                        continue;
                    }
                };
            self.internal_channel
                .send(Event::StoppedAdvertising { discoverable })
                .await;

            if self.min_connection_interval.is_some() || self.max_connection_interval.is_some() {
                debug!("Updating connection parameters");
                let mut conn_params = RequestedConnParams::default();
                if let Some(v) = self.min_connection_interval {
                    conn_params.min_connection_interval = v;
                }
                if let Some(v) = self.max_connection_interval {
                    conn_params.max_connection_interval = v;
                }
                if let Some(v) = self.supervision_timeout {
                    conn_params.supervision_timeout = v;
                }
                let result = new_connection
                    .update_connection_params(ble_stack, &conn_params)
                    .await;
                if result.is_err() {
                    error!("Failed to update connection parameters");
                }
            }

            let connection_handle = new_connection.handle();
            let new_connection = match new_connection.with_attribute_server(runner.server) {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to add attribute server: {}", e);
                    if let Some(connection) = ble_stack.get_connected_handle(connection_handle) {
                        connection.disconnect();
                    }
                    Timer::after_millis(250).await;
                    continue;
                }
            };
            if let Err(e) = new_connection.raw().set_bondable(true) {
                error!("Failed to set connection as bondable: {}", e);
            }
            if let Err(e) = new_connection.raw().request_security() {
                error!("Failed to request security: {}", e);
            }
            *runner.connection.write().await = Some(new_connection);

            info!("BLE connected");
            self.internal_channel.send(Event::Connected).await;

            #[allow(clippy::while_let_loop)]
            loop {
                match self
                    .handle_next_connection_event(runner, profile_index)
                    .await
                {
                    ConnectionStatusResult::Connected => {}
                    ConnectionStatusResult::Disconnected => break,
                };
            }

            self.internal_channel.send(Event::Disconnected).await;
        }
    }

    async fn handle_next_connection_event<'server, Storage>(
        &self,
        runner: &Runner<
            'server,
            Storage,
            TxMessage::MessageService,
            RxMessage::MessageService,
            CONN_MAX,
        >,
        profile_index: u8,
    ) -> ConnectionStatusResult
    where
        Storage: storage::Storage,
    {
        let event = {
            let connection = runner.connection.read().await;
            let connection = connection.as_ref().unwrap();
            if !connection.raw().is_connected() {
                debug!("BLE is not connected");
                return ConnectionStatusResult::Disconnected;
            }
            connection.next().await
        };
        match event {
            GattConnectionEvent::Disconnected { reason } => {
                info!("BLE disconnected (reason: {})", reason);
                if reason.to_result() == Err(bt_hci::param::Error::AUTHENTICATION_FAILURE) {
                    // The stored bond no longer matches the peer; drop it so the next
                    // connection performs a fresh pairing instead of failing forever.
                    warn!("Clearing stale bond for profile {}", profile_index);
                    let mut write_transaction = runner.storage.write_transaction().await;
                    if let Err(e) = write_transaction
                        .delete::<StoredBondInformation>(profile_index)
                        .await
                    {
                        #[cfg(feature = "defmt")]
                        let e = defmt::Debug2Format(&e);
                        error!("Failed to delete bond info from storage: {}", e);
                    }
                    if let Err(e) = write_transaction.commit().await {
                        #[cfg(feature = "defmt")]
                        let e = defmt::Debug2Format(&e);
                        error!("Failed to commit to storage: {}", e);
                    }
                    runner.profile_bond_infos.lock().await[profile_index as usize] = None;
                }
                return ConnectionStatusResult::Disconnected;
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
                    return ConnectionStatusResult::Connected;
                };
                debug!("Received bond information: {}", bond);

                if let Some(stored_bond_info) =
                    &runner.profile_bond_infos.lock().await[profile_index as usize]
                    && !stored_bond_info.identity.match_identity(&bond.identity)
                {
                    error!("Stored bond info does not match identity");
                    return ConnectionStatusResult::Disconnected;
                }

                debug!("Writing bond info to flash");
                {
                    let mut write_transaction = runner.storage.write_transaction().await;
                    if let Err(e) = write_transaction
                        .write(profile_index, StoredBondInformation::from_ref(&bond))
                        .await
                    {
                        #[cfg(feature = "defmt")]
                        let e = defmt::Debug2Format(&e);
                        error!("Failed to write bond info to flash: {}", e);
                    }
                    if let Err(e) = write_transaction.commit().await {
                        #[cfg(feature = "defmt")]
                        let e = defmt::Debug2Format(&e);
                        error!("Failed to commit to database: {}", e);
                    }
                }
                runner.profile_bond_infos.lock().await[profile_index as usize] = Some(bond);
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
                debug!("Received GATT event: {}", event);
                if let GattEvent::Write(write_event) = &event {
                    let connection = runner.connection.read().await;
                    let connection = connection.as_ref().unwrap();
                    if let Some(message) = runner
                        .rx_message_service
                        .receive(write_event, connection)
                        .await
                    {
                        self.rx_channel.send(message).await;
                    }
                }
                match event.accept() {
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
            GattConnectionEvent::SubratingParamsUpdated { .. } => {
                debug!("Received SubratingParamsUpdated event");
            }
            GattConnectionEvent::FrameSpaceUpdated { .. } => {
                debug!("Received FrameSpaceUpdated event");
            }
            GattConnectionEvent::ConnectionRateChanged { .. } => {
                debug!("Received ConnectionRateChanged event");
            }
            GattConnectionEvent::BondLost => {
                debug!("Received BondLost event");
                runner.remove_bond(profile_index).await;
            }
            GattConnectionEvent::Encrypted { bond, .. } => {
                debug!("Received Encrypted event");
                if let Some(bond) = bond {
                    debug!("Received bond information: {}", bond);
                }
            }
            GattConnectionEvent::OobRequest => {
                debug!("Received OobRequest event");
            }
        }
        ConnectionStatusResult::Connected
    }
}
