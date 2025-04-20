use crate::mcu::{Mcu, McuBle};
use crate::util::channel::Channel;
use crate::util::{debug, error, info, unwrap};
use crate::{Address, internal};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::future::Future;
use core::mem::transmute;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};
use embassy_executor::Spawner;
use embassy_executor::raw::TaskStorage;
use embassy_futures::join::join;
use embassy_futures::select::{select, select3};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::Timer;
use trouble_host::gatt::{GattClient, GattConnectionEvent, GattEvent};
use trouble_host::prelude::{
    AddrKind, Advertisement, AdvertisementParameters, AsGatt, BdAddr, Characteristic,
    ConnectConfig, FromGatt, ScanConfig, Uuid,
};
use trouble_host::types::gatt_traits::FromGattError;

pub enum TransportConfig {
    Central {
        peripheral_addresses: &'static [Address],
    },
    Peripheral {
        central_address: Address,
    },
}

const SERVICE_UUID: Uuid = Uuid::Uuid128([
    0x2e, 0x51, 0x03, 0x5f, 0xd3, 0x9b, 0x41, 0xfe, 0x8b, 0x1b, 0x70, 0xa5, 0x3e, 0x58, 0xa9, 0x85,
]);
const MESSAGE_TO_CENTRAL_CHARACTERISTIC_UUID: Uuid = Uuid::Uuid128([
    0xf7, 0x32, 0xa6, 0x4b, 0x06, 0xbb, 0x4f, 0xec, 0x94, 0xac, 0x7a, 0xbf, 0xe1, 0x11, 0x9a, 0xc8,
]);
const MESSAGE_TO_PERIPHERAL_CHARACTERISTIC_UUID: Uuid = Uuid::Uuid128([
    0x3d, 0x90, 0x87, 0x1d, 0xe7, 0xd9, 0x40, 0x64, 0xb3, 0x74, 0x6b, 0x24, 0x80, 0x71, 0x4e, 0xf6,
]);

#[derive(Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct Message(Vec<u8>);

impl AsGatt for Message {
    const MIN_SIZE: usize = 0;
    const MAX_SIZE: usize = 1024;

    fn as_gatt(&self) -> &[u8] {
        &self.0
    }
}

impl FromGatt for Message {
    fn from_gatt(data: &[u8]) -> Result<Self, FromGattError> {
        Ok(Message(Vec::from(data)))
    }
}

mod peripheral {
    #![allow(clippy::useless_conversion)] // Produced by the macros from trouble_host

    use super::{
        MESSAGE_TO_CENTRAL_CHARACTERISTIC_UUID, MESSAGE_TO_PERIPHERAL_CHARACTERISTIC_UUID, Message,
        SERVICE_UUID,
    };
    use trouble_host::prelude::*;

    #[gatt_service(uuid = SERVICE_UUID)]
    pub struct Service {
        #[characteristic(uuid = MESSAGE_TO_CENTRAL_CHARACTERISTIC_UUID, read, notify)]
        pub message_to_central: Message,
        #[characteristic(uuid = MESSAGE_TO_PERIPHERAL_CHARACTERISTIC_UUID, write_without_response)]
        pub message_to_peripheral: Message,
    }

    #[gatt_server]
    pub struct Server {
        pub service: Service,
    }
}

static SEND_CHANNEL: Channel<CriticalSectionRawMutex, Message> = Channel::new();
static RECV_CHANNEL: Channel<CriticalSectionRawMutex, Message> = Channel::new();
static IS_CONNECTED: AtomicBool = AtomicBool::new(false);

#[non_exhaustive]
pub struct Transport {}

impl internal::Transport for Transport {
    fn send(&self, message_bytes: &[u8]) {
        if IS_CONNECTED.load(Ordering::Acquire) {
            SEND_CHANNEL.send(Message(Vec::from(message_bytes)));
        }
    }

    fn receive(&self) -> Pin<Box<dyn Future<Output = Vec<u8>> + '_>> {
        Box::pin(async { RECV_CHANNEL.receive().await.0 })
    }
}

impl<M: Mcu + McuBle> internal::TransportConfig<M> for internal::ble::TransportConfig {
    type Transport = Transport;

    async fn init(self, mcu: &'static M, _address: Address, spawner: Spawner) -> Self::Transport {
        match self {
            Self::Central {
                peripheral_addresses,
            } => {
                let task_storage = Box::leak(Box::new(TaskStorage::new()));
                let task = task_storage.spawn(move || central(mcu, peripheral_addresses));
                unwrap!(spawner.spawn(task));
            }
            Self::Peripheral { central_address } => {
                let task_storage = Box::leak(Box::new(TaskStorage::new()));
                let task = task_storage.spawn(move || peripheral(mcu, central_address));
                unwrap!(spawner.spawn(task))
            }
        }
        Transport {}
    }
}

async fn central<M: Mcu + McuBle>(mcu: &'static M, peripheral_addresses: &'static [Address]) {
    let ble_stack = mcu.ble_stack();
    let mut host = ble_stack.build();

    let filter_accept_list = peripheral_addresses
        .iter()
        .map(|address| {
            (AddrKind::RANDOM, unsafe {
                transmute::<&[u8; 6], &BdAddr>(&address.0)
            })
        })
        .collect::<Vec<_>>();
    let config = ConnectConfig {
        scan_config: ScanConfig {
            filter_accept_list: filter_accept_list.as_slice(),
            ..Default::default()
        },
        connect_params: Default::default(),
    };

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

    let connect = async {
        loop {
            debug!("Looking for BLE connection to peripheral");
            let connection = match host.central.connect(&config).await {
                Ok(v) => v,
                Err(e) => {
                    #[cfg(feature = "defmt")]
                    let e = defmt::Debug2Format(&e);
                    error!("Failed to connect: {}", e);
                    Timer::after_secs(1).await;
                    continue;
                }
            };
            let client = match GattClient::<_, 1, 72>::new(ble_stack, &connection).await {
                Ok(v) => v,
                Err(e) => {
                    #[cfg(feature = "defmt")]
                    let e = defmt::Debug2Format(&e);
                    error!("Failed to create GATT client: {}", e);
                    Timer::after_secs(1).await;
                    continue;
                }
            };
            info!("BLE connected to peripheral");
            IS_CONNECTED.store(true, Ordering::Release);

            let check_connection = async {
                loop {
                    if !connection.is_connected() {
                        info!("BLE disconnected");
                        break;
                    }
                    Timer::after_secs(1).await;
                }
            };

            let client_task = async {
                if let Err(e) = client.task().await {
                    #[cfg(feature = "defmt")]
                    let e = defmt::Debug2Format(&e);
                    error!("GATT client task failed: {}", e);
                    Timer::after_secs(1).await;
                }
            };

            let handle_messages = async || {
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
                            let message = Message(Vec::from(message.as_ref()));
                            debug!("Received message from peripheral: {}", message);
                            RECV_CHANNEL.send(message);
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
                            .write_characteristic_without_response(
                                &message_to_peripheral,
                                &message.0,
                            )
                            .await
                        {
                            #[cfg(feature = "defmt")]
                            let e = defmt::Debug2Format(&e);
                            error!("Failed to write characteristic: {}", e);
                        }
                    }
                };
                select(receive, send).await;
            };

            select3(check_connection, client_task, handle_messages()).await;

            IS_CONNECTED.store(false, Ordering::Release);
        }
    };

    join(run, connect).await;
}

async fn peripheral<M: Mcu + McuBle>(mcu: &'static M, central_address: Address) {
    let ble_stack = mcu.ble_stack();
    let mut host = ble_stack.build();

    let adv_params = AdvertisementParameters::default();
    let adv = Advertisement::ConnectableNonscannableDirected {
        peer: trouble_host::Address::random(central_address.0),
    };

    let server = unwrap!(peripheral::Server::new_default("lokey_peripheral"));

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

    let connect = async {
        loop {
            info!("Starting BLE advertisement");
            let advertiser = match host.peripheral.advertise(&adv_params, adv).await {
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
            IS_CONNECTED.store(true, Ordering::Release);
            info!("BLE connected to central");

            let receive = async {
                loop {
                    match connection.next().await {
                        GattConnectionEvent::Disconnected { reason } => {
                            info!("BLE disconnected (reason: {})", reason);
                            break;
                        }
                        GattConnectionEvent::Gatt { event } => match event {
                            Ok(event) => {
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
                                            let message = Message(Vec::from(write_event.data()));
                                            debug!("Received message from central: {}", message);
                                            RECV_CHANNEL.send(message);
                                        }
                                    }
                                }
                                match event.accept() {
                                    Ok(reply) => reply.send().await,
                                    Err(error) => error!("Failed to handle event: {}", error),
                                }
                            }
                            Err(error) => error!("Error processing event: {}", error),
                        },
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
                        .notify(&connection, &message)
                        .await
                    {
                        error!("Failed to send value: {}", e);
                    }
                }
            };

            select(receive, send).await;

            IS_CONNECTED.store(false, Ordering::Release);
        }
    };

    join(run, connect).await;
}
