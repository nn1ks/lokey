use crate::BleStack;
use crate::internal::{Message, SERVICE_UUID128};
use arrayvec::ArrayVec;
use core::sync::atomic::Ordering;
use embassy_futures::select::select;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;
use generic_array::GenericArray;
use lokey::internal::MAX_MESSAGE_SIZE_WITH_TAG;
use lokey::storage::{self, BASE_TAG_SIZE, Entry, ReadTransaction, WriteTransaction};
use lokey::util::{debug, error, info, unwrap};
use lokey::{Address, AnyState, Context, McuCreator, internal};
use portable_atomic::AtomicBool;
use trouble_host::BondInformation;
use trouble_host::advertise::{AdStructure, BR_EDR_NOT_SUPPORTED};
use trouble_host::gatt::{GattConnectionEvent, GattEvent};
use trouble_host::prelude::{Advertisement, AdvertisementParameters};

static SEND_CHANNEL: Channel<CriticalSectionRawMutex, Message, 1> = Channel::new();
static RECV_CHANNEL: Channel<CriticalSectionRawMutex, Message, 1> = Channel::new();
static IS_CONNECTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PeripheralTransportConfig;

pub struct PeripheralTransport<Mcu: 'static> {
    mcu: &'static Mcu,
}

impl<Mcu> internal::Transport for PeripheralTransport<Mcu>
where
    Mcu: BleStack + 'static,
{
    type Config = PeripheralTransportConfig;
    type Mcu = Mcu;

    async fn create<Storage>(
        _: Self::Config,
        _: Address,
        mcu: &'static Self::Mcu,
        _: &'static Storage,
    ) -> Self {
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
        let mut ble_host_peripheral = ble_stack.peripheral();

        let adv_params = AdvertisementParameters::default();
        let adv_data = build_adv_data();

        let server = unwrap!(gatt::Server::new_default("lokey-peripheral"));

        loop {
            let bond = self.read_bond_from_storage(context.storage).await;

            if let Some(bond) = &bond
                && let Err(e) = ble_stack.add_bond_information(bond.0.clone())
            {
                error!("Failed to add bond info to BLE stack: {}", e);
            }

            let adv = match &bond {
                Some(bond) => Advertisement::ConnectableNonscannableDirected {
                    peer: bond.0.identity.addr,
                },
                None => Advertisement::ConnectableScannableUndirected {
                    adv_data: &adv_data,
                    scan_data: &[],
                },
            };

            info!("Starting BLE advertisement");
            let advertiser = match ble_host_peripheral.advertise(&adv_params, adv).await {
                Ok(v) => v,
                Err(e) => {
                    #[cfg(feature = "defmt")]
                    let e = defmt::Debug2Format(&e);
                    error!("Failed to advertise: {}", e);
                    Timer::after_secs(1).await;
                    continue;
                }
            };
            let connection = match advertiser.accept().await {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                    Timer::after_secs(1).await;
                    continue;
                }
            };
            let connection = match connection.with_attribute_server(&server) {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to add attribute server to connection: {}", e);
                    Timer::after_secs(1).await;
                    continue;
                }
            };
            if let Err(e) = connection.raw().set_bondable(bond.is_none()) {
                error!("Failed to set connection as bondable: {}", e);
            }
            IS_CONNECTED.store(true, Ordering::Release);
            info!("BLE connected to central");

            let receive = async {
                loop {
                    match connection.next().await {
                        GattConnectionEvent::PairingComplete {
                            security_level,
                            bond: received_bond,
                        } => {
                            debug!(
                                "Received PairingComplete event with security level {}",
                                security_level
                            );
                            debug!("Received bond information: {}", received_bond);
                            if let Some(received_bond) = received_bond {
                                if let Some(bond) = &bond
                                    && !bond.0.identity.match_identity(&received_bond.identity)
                                {
                                    error!("Stored bond info does not match identity");
                                    break;
                                }
                                self.write_bond_to_storage(
                                    context.storage,
                                    &CentralBond(received_bond),
                                )
                                .await;
                            }
                        }
                        GattConnectionEvent::PairingFailed(e) => {
                            error!("Pairing failed: {}", e);
                        }
                        GattConnectionEvent::Disconnected { reason } => {
                            info!("BLE disconnected (reason: {})", reason);
                            break;
                        }
                        GattConnectionEvent::RequestConnectionParams(params) => {
                            debug!("Received RequestConnectionParams event: {}", params);
                            if let Err(e) = params.accept(None, ble_stack).await {
                                error!("Failed to accept connection params request: {}", e);
                            }
                        }
                        GattConnectionEvent::Gatt { event } => {
                            debug!("Received GATT event");
                            match &event {
                                GattEvent::Read(read_event) => {
                                    debug!("GATT read event: {}", read_event.handle())
                                }
                                GattEvent::Write(write_event) => {
                                    debug!("GATT write event: {}", write_event.handle());
                                    if write_event.handle()
                                        == server.service.message_to_peripheral.handle
                                    {
                                        write_event.with_data(|_, data| {
                                            debug!("Received message from central: {}", data)
                                        });
                                        match write_event
                                            .with_data(|_, data| ArrayVec::try_from(data))
                                        {
                                            Ok(array) => {
                                                RECV_CHANNEL.send(Message(array)).await;
                                            }
                                            Err(_) => {
                                                error!(
                                                    "Reiceved message exceeds configured max message size"
                                                )
                                            }
                                        };
                                    }
                                }
                                GattEvent::NotAllowed(_) => debug!("GATT not allowed event"),
                                GattEvent::Other(_) => debug!("GATT other event"),
                            }
                            match event.accept() {
                                Ok(reply) => reply.send().await,
                                Err(error) => error!("Failed to handle event: {}", error),
                            }
                        }
                        _ => {}
                    }
                }
            };

            let send = async {
                loop {
                    let message = SEND_CHANNEL.receive().await;
                    debug!("Sending message to central: {}", message);
                    if let Err(e) = server
                        .service
                        .message_to_central
                        .notify(&connection, &message, true)
                        .await
                    {
                        error!("Failed to send value: {}", e);
                    }
                }
            };

            select(receive, send).await;

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

impl<Mcu> PeripheralTransport<Mcu>
where
    Mcu: BleStack + 'static,
{
    async fn read_bond_from_storage<Storage>(
        &self,
        storage: &'static Storage,
    ) -> Option<CentralBond>
    where
        Storage: storage::Storage,
    {
        let read_transaction = storage.read_transaction().await;
        match read_transaction.read::<CentralBond>(()).await {
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

    async fn write_bond_to_storage<Storage>(&self, storage: &'static Storage, bond: &CentralBond)
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
}

mod gatt {
    #![allow(clippy::useless_conversion, clippy::needless_borrows_for_generic_args)] // Produced by the macros from trouble_host

    use crate::internal::{
        MESSAGE_TO_CENTRAL_CHARACTERISTIC_UUID, MESSAGE_TO_PERIPHERAL_CHARACTERISTIC_UUID, Message,
        SERVICE_UUID,
    };
    use trouble_host::prelude::*;

    #[gatt_service(uuid = SERVICE_UUID)]
    pub struct Service {
        #[characteristic(
            uuid = MESSAGE_TO_CENTRAL_CHARACTERISTIC_UUID,
            read,
            notify,
            permissions(encrypted)
        )]
        pub message_to_central: Message,
        #[characteristic(
            uuid = MESSAGE_TO_PERIPHERAL_CHARACTERISTIC_UUID,
            write_without_response,
            permissions(encrypted)
        )]
        pub message_to_peripheral: Message,
    }

    #[gatt_server]
    pub struct Server {
        pub service: Service,
    }
}

fn build_adv_data() -> ArrayVec<u8, 31> {
    let mut adv_data = [0; 31];
    let ad_structure = &[
        AdStructure::Flags(BR_EDR_NOT_SUPPORTED),
        AdStructure::CompleteServiceUuids128(&[SERVICE_UUID128]),
    ];
    let adv_data_len = unwrap!(AdStructure::encode_slice(ad_structure, &mut adv_data));
    ArrayVec::try_from(&adv_data[..adv_data_len]).expect("adv data too large")
}

struct CentralBond(BondInformation);

impl Entry for CentralBond {
    type Size = typenum::U41;
    type TagParam = ();

    const BASE_TAG: [u8; BASE_TAG_SIZE] = [0x00, 0x00, 0x00, 0x03];

    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self>
    where
        Self: Sized,
    {
        crate::bond_information::from_bytes(bytes).map(CentralBond)
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        crate::bond_information::to_bytes(&self.0)
    }
}
