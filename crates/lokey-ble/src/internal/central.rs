use crate::BleStack;
use crate::internal::{
    MESSAGE_TO_CENTRAL_CHARACTERISTIC_UUID, MESSAGE_TO_PERIPHERAL_CHARACTERISTIC_UUID, Message,
    SERVICE_UUID, SERVICE_UUID128,
};
use arrayvec::ArrayVec;
use bt_hci::param::{LeAdvEventKind, LeAdvReportsIter};
use core::sync::atomic::Ordering;
use embassy_futures::select::{select, select3};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, TimeoutError, Timer, WithTimeout};
use generic_array::GenericArray;
use lokey::internal::{self, MAX_MESSAGE_SIZE_WITH_TAG};
use lokey::storage::{self, Entry, ReadTransaction, WriteTransaction};
use lokey::util::{debug, error, info};
use lokey::{AnyState, Context, McuCreator};
use portable_atomic::AtomicBool;
use trouble_host::advertise::AdStructure;
use trouble_host::connection::{Connection, ConnectionEvent};
use trouble_host::gatt::GattClient;
use trouble_host::prelude::{
    Characteristic, ConnectConfig, DefaultPacketPool, EventHandler, ScanConfig,
};
use trouble_host::scan::Scanner;
use trouble_host::{Address, BondInformation};

const MAX_SERVICES: usize = 1;

static SEND_CHANNEL: Channel<CriticalSectionRawMutex, Message, 1> = Channel::new();
static RECV_CHANNEL: Channel<CriticalSectionRawMutex, Message, 1> = Channel::new();
static IS_CONNECTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CentralTransportConfig;

pub struct CentralTransport<Mcu: 'static> {
    mcu: &'static Mcu,
}

impl<Mcu: BleStack + 'static> internal::Transport for CentralTransport<Mcu> {
    type Config = CentralTransportConfig;
    type Mcu = Mcu;

    async fn create<Storage>(
        _: Self::Config,
        _: lokey::Address,
        mcu: &'static Self::Mcu,
        _: &'static Storage,
    ) -> Self
    where
        Storage: storage::Storage,
    {
        Self { mcu }
    }

    async fn run<Device, Transports, State>(&self, context: Context<Device, Transports, State>)
    where
        Device: lokey::Device<Mcu = Self::Mcu>,
        Transports: lokey::Transports<Self::Mcu>,
        State: AnyState,
        Self::Mcu: McuCreator<Device::StorageConfig>,
    {
        let ble_stack = self.mcu.ble_stack();
        let mut ble_host_central = ble_stack.central();

        if !self.mcu.register_ble_event_handler(&AdvEventHandler) {
            error!("Failed to register BLE event handler: BLE stack already has an event handler");
        }

        loop {
            let bond = self.read_bond_from_storage(context.storage).await;

            if let Some(bond) = &bond
                && let Err(e) = ble_stack.add_bond_information(bond.0.clone())
            {
                error!("Failed to add bond info to BLE stack: {}", e);
            }

            let peer_address = match &bond {
                Some(bond) => bond.0.identity.addr,
                None => {
                    debug!("Starting BLE scan");
                    let mut scanner = Scanner::new(&mut ble_host_central);
                    let scan_config = ScanConfig {
                        filter_accept_list: &[],
                        interval: Duration::from_secs(3),
                        ..Default::default()
                    };
                    let session = match scanner.scan(&scan_config).await {
                        Ok(v) => v,
                        Err(e) => {
                            error!("Failed to start scanning: {}", e);
                            Timer::after_secs(5).await;
                            continue;
                        }
                    };
                    let peer_address = ADV_ADDRESS_SIGNAL.wait().await;
                    debug!("Found advertised peer: {}", peer_address);
                    session.stop().await;
                    ADV_ADDRESS_SIGNAL.reset();
                    peer_address
                }
            };

            let config = ConnectConfig {
                scan_config: ScanConfig {
                    filter_accept_list: &[peer_address],
                    ..Default::default()
                },
                connect_params: Default::default(),
            };

            debug!("Looking for BLE connection to peripheral");
            let connection = match ble_host_central.connect(&config).await {
                Ok(v) => v,
                Err(e) => {
                    #[cfg(feature = "defmt")]
                    let e = defmt::Debug2Format(&e);
                    error!("Failed to connect: {}", e);
                    Timer::after_secs(1).await;
                    continue;
                }
            };
            if let Err(e) = connection.set_bondable(bond.is_none()) {
                error!("Failed to set bondable: {}", e);
            }
            if let Err(e) = connection.request_security() {
                error!("Failed to request security: {}", e);
            }

            if bond.is_none() {
                debug!("Pairing peripheral device");
                loop {
                    match connection.next().await {
                        ConnectionEvent::PairingComplete {
                            security_level,
                            bond: received_bond,
                        } => {
                            debug!(
                                "Received PairingComplete event with security level {}",
                                security_level
                            );
                            debug!("Received bond information: {}", received_bond);
                            if let Some(received_bond) = received_bond {
                                self.write_bond_to_storage(
                                    context.storage,
                                    &PeripheralBond(received_bond),
                                )
                                .await;
                            }
                            break;
                        }
                        ConnectionEvent::PairingFailed(e) => {
                            error!("Pairing failed: {}", e);
                            break;
                        }
                        ConnectionEvent::Disconnected { reason } => {
                            error!("BLE disconnected (reason: {})", reason);
                            break;
                        }
                        ConnectionEvent::RequestConnectionParams(params) => {
                            debug!("Received RequestConnectionParams event: {}", params);
                            if let Err(e) = params.accept(None, ble_stack).await {
                                error!("Failed to accept connection params request: {}", e);
                            }
                        }
                        _ => {}
                    }
                }
            }

            let client =
                match GattClient::<_, DefaultPacketPool, MAX_SERVICES>::new(ble_stack, &connection)
                    .with_timeout(Duration::from_secs(1))
                    .await
                {
                    Ok(Ok(v)) => v,
                    Ok(Err(e)) => {
                        #[cfg(feature = "defmt")]
                        let e = defmt::Debug2Format(&e);
                        error!("Failed to create GATT client: {}", e);
                        Timer::after_secs(1).await;
                        continue;
                    }
                    Err(TimeoutError) => {
                        error!("Reached timeout while creating GATT client");
                        continue;
                    }
                };
            info!("BLE connected to peripheral");
            IS_CONNECTED.store(true, Ordering::Release);

            select3(
                self.check_connection(&connection),
                self.client_task(&client),
                self.handle_messages(&client),
            )
            .await;

            IS_CONNECTED.store(false, Ordering::Release);
        }
    }

    async fn send(&self, message_bytes: &[u8]) {
        if IS_CONNECTED.load(Ordering::Acquire) {
            match ArrayVec::try_from(message_bytes) {
                Ok(array) => SEND_CHANNEL.send(Message(array)).await,
                Err(_) => error!("Size of message exceeds configured max message size"),
            };
        }
    }

    async fn receive(&self, buf: &mut [u8]) -> usize {
        let array = RECV_CHANNEL.receive().await.0;
        if buf.len() < MAX_MESSAGE_SIZE_WITH_TAG {
            panic!("Provided buffer is smaller than configured max message size");
        }
        let len = array.len();
        for (i, value) in array.into_iter().enumerate() {
            buf[i] = value;
        }
        len
    }
}

impl<Mcu> CentralTransport<Mcu>
where
    Mcu: BleStack + 'static,
{
    async fn read_bond_from_storage<Storage>(
        &self,
        storage: &'static Storage,
    ) -> Option<PeripheralBond>
    where
        Storage: storage::Storage,
    {
        let read_transaction = storage.read_transaction().await;
        match read_transaction.read::<PeripheralBond>(()).await {
            Ok(v) => {
                info!("Found existing central bond info: {}", v.0);
                Some(v)
            }
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Debug2Format(&e);
                error!("Failed to read peripheral bond from storage: {}", e);
                None
            }
        }
    }

    async fn write_bond_to_storage<Storage>(&self, storage: &'static Storage, bond: &PeripheralBond)
    where
        Storage: storage::Storage,
    {
        debug!("Storing bond information in storage");
        let mut write_transaction = storage.write_transaction().await;
        if let Err(e) = write_transaction.write((), bond).await {
            #[cfg(feature = "defmt")]
            let e = defmt::Debug2Format(&e);
            error!("Failed to write bond to database: {}", e);
        }
        if let Err(e) = write_transaction.commit().await {
            #[cfg(feature = "defmt")]
            let e = defmt::Debug2Format(&e);
            error!("Failed to commit to database: {}", e);
        }
    }

    async fn check_connection(&self, connection: &Connection<'_, DefaultPacketPool>) {
        loop {
            if !connection.is_connected() {
                info!("BLE disconnected");
                break;
            }
            Timer::after_secs(1).await;
        }
    }

    async fn client_task(
        &self,
        client: &GattClient<'_, Mcu::Controller, DefaultPacketPool, MAX_SERVICES>,
    ) {
        if let Err(e) = client.task().await {
            #[cfg(feature = "defmt")]
            let e = defmt::Debug2Format(&e);
            error!("GATT client task failed: {}", e);
            Timer::after_secs(1).await;
        }
    }

    async fn handle_messages(
        &self,
        client: &GattClient<'_, Mcu::Controller, DefaultPacketPool, MAX_SERVICES>,
    ) {
        let services = match client.services_by_uuid(&SERVICE_UUID).await {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Debug2Format(&e);
                error!("Failed to discover service: {}", e);
                return;
            }
        };
        let service = match services.into_iter().next() {
            Some(v) => v,
            None => {
                error!("Service not found");
                return;
            }
        };

        let message_to_central: Characteristic<Message> = match client
            .characteristic_by_uuid(&service, &MESSAGE_TO_CENTRAL_CHARACTERISTIC_UUID)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Debug2Format(&e);
                error!("Failed to discover characteristic: {}", e);
                return;
            }
        };
        let message_to_peripheral: Characteristic<Message> = match client
            .characteristic_by_uuid(&service, &MESSAGE_TO_PERIPHERAL_CHARACTERISTIC_UUID)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "defmt")]
                let e = defmt::Debug2Format(&e);
                error!("Failed to discover characteristic: {}", e);
                return;
            }
        };

        let receive = async {
            match client.subscribe(&message_to_central, false).await {
                Ok(mut listener) => loop {
                    let message = listener.next().await;
                    let message = message.as_ref();
                    debug!("Received message from peripheral: {:?}", message);
                    match ArrayVec::try_from(message) {
                        Ok(array) => {
                            RECV_CHANNEL.send(Message(array)).await;
                        }
                        Err(_) => {
                            error!("Reiceved message exceeds configured max message size")
                        }
                    };
                },
                Err(e) => {
                    #[cfg(feature = "defmt")]
                    let e = defmt::Debug2Format(&e);
                    error!("Failed to subscribe to client: {}", e);
                }
            }
        };
        let send = async {
            loop {
                let message = SEND_CHANNEL.receive().await;
                debug!("Sending message to peripheral: {}", message);
                if let Err(e) = client
                    .write_characteristic_without_response(&message_to_peripheral, &message.0)
                    .await
                {
                    #[cfg(feature = "defmt")]
                    let e = defmt::Debug2Format(&e);
                    error!("Failed to write characteristic: {}", e);
                }
            }
        };
        select(receive, send).await;
    }
}

struct PeripheralBond(BondInformation);

impl Entry for PeripheralBond {
    type Size = typenum::U41;
    type TagParam = ();

    const BASE_TAG: [u8; 4] = [0x00, 0x00, 0x00, 0x04];

    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self>
    where
        Self: Sized,
    {
        crate::bond_information::from_bytes(bytes).map(PeripheralBond)
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        crate::bond_information::to_bytes(&self.0)
    }
}

static ADV_ADDRESS_SIGNAL: Signal<CriticalSectionRawMutex, Address> = Signal::new();

struct AdvEventHandler;

impl AdvEventHandler {
    fn contains_service_uuid(data: &[u8]) -> bool {
        AdStructure::decode(data).any(|ad_structure| match ad_structure {
            Ok(
                AdStructure::CompleteServiceUuids128(uuids)
                | AdStructure::IncompleteServiceUuids128(uuids),
            ) => uuids.contains(&SERVICE_UUID128),
            _ => false,
        })
    }
}

impl EventHandler for AdvEventHandler {
    fn on_adv_reports(&self, reports: LeAdvReportsIter<'_>) {
        for report in reports {
            let report = match report {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to get report: {}", e);
                    continue;
                }
            };
            if AdvEventHandler::contains_service_uuid(report.data)
                && report.event_kind == LeAdvEventKind::AdvInd
                || report.event_kind == LeAdvEventKind::AdvDirectInd
            {
                debug!("Found report with internal service UUID: {}", report);
                let address = Address {
                    kind: report.addr_kind,
                    addr: report.addr,
                };
                ADV_ADDRESS_SIGNAL.signal(address)
            }
        }
    }
}
