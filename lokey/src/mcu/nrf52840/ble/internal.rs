use super::BLE_ADDRESS_WAS_SET;
use crate::util::{channel::Channel, unwrap};
use crate::{internal, mcu::Nrf52840};
use alloc::{boxed::Box, vec::Vec};
use core::{future::Future, pin::Pin, sync::atomic::Ordering};
#[cfg(feature = "defmt")]
use defmt::{debug, error, info, warn};
use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use nrf_softdevice::Softdevice;
use nrf_softdevice::ble::{
    Address, AddressType, GattValue, central, gatt_client, gatt_server, peripheral,
};
use nrf_softdevice::{gatt_client, gatt_server, gatt_service};

pub struct Message(Vec<u8>);

impl GattValue for Message {
    const MIN_SIZE: usize = 0;
    const MAX_SIZE: usize = 128;

    fn from_gatt(data: &[u8]) -> Self {
        Self(Vec::from(data))
    }

    fn to_gatt(&self) -> &[u8] {
        &self.0
    }
}

#[gatt_server]
struct Server {
    service: Service,
}

#[gatt_service(uuid = "2e51035f-d39b-41fe-8b1b-70a53e58a985")]
struct Service {
    #[characteristic(uuid = "f732a64b-06bb-4fec-94ac-7abfe1119ac8", read, notify)]
    message_to_central: Message,
    #[characteristic(uuid = "3d90871d-e7d9-4064-b374-6b2480714ef6", write_without_response)]
    message_to_peripheral: Message,
}

#[gatt_client(uuid = "2e51035f-d39b-41fe-8b1b-70a53e58a985")]
#[derive(Clone)]
struct Client {
    #[characteristic(uuid = "f732a64b-06bb-4fec-94ac-7abfe1119ac8", read, notify)]
    message_to_central: Message,
    #[characteristic(uuid = "3d90871d-e7d9-4064-b374-6b2480714ef6", write)]
    message_to_peripheral: Message,
}

static SEND_CHANNEL: Channel<CriticalSectionRawMutex, Message> = Channel::new();
static RECV_CHANNEL: Channel<CriticalSectionRawMutex, Message> = Channel::new();

#[non_exhaustive]
pub struct Transport {}

impl internal::Transport for Transport {
    fn send(&self, message_bytes: &[u8]) {
        let message = Message(Vec::from(message_bytes));
        SEND_CHANNEL.send(message);
    }

    fn receive(&self) -> Pin<Box<dyn Future<Output = Vec<u8>> + '_>> {
        Box::pin(async { RECV_CHANNEL.receive().await.0 })
    }
}

impl internal::TransportConfig<Nrf52840> for internal::ble::TransportConfig {
    type Transport = Transport;

    async fn init(self, mcu: &'static Nrf52840, spawner: Spawner) -> Self::Transport {
        let softdevice: &'static mut Softdevice = unsafe { &mut *mcu.softdevice.get() };
        match self {
            Self::Central {
                address,
                peripheral_addresses,
            } => {
                unwrap!(spawner.spawn(central(softdevice, address, peripheral_addresses)));
            }
            Self::Peripheral {
                address,
                central_address,
            } => {
                let server = unwrap!(Server::new(softdevice));
                unwrap!(spawner.spawn(peripheral(softdevice, server, address, central_address)));
            }
        }
        Transport {}
    }
}

#[embassy_executor::task]
async fn central(
    softdevice: &'static Softdevice,
    central_address: [u8; 6],
    peripheral_addresses: &'static [[u8; 6]],
) {
    if !BLE_ADDRESS_WAS_SET.load(Ordering::SeqCst) {
        nrf_softdevice::ble::set_address(
            softdevice,
            &Address::new(AddressType::RandomStatic, central_address),
        );
        BLE_ADDRESS_WAS_SET.store(true, Ordering::SeqCst);
    }

    let mut connect_config = central::ConnectConfig::default();

    let mut whitelisted_addresses = Vec::new();
    for address in peripheral_addresses {
        whitelisted_addresses.push(Address::new(AddressType::RandomStatic, *address));
    }
    let whitelisted_addresses = whitelisted_addresses.iter().collect::<Vec<_>>();
    connect_config.scan_config.whitelist = Some(&whitelisted_addresses);

    loop {
        debug!("Connecting to peripheral...");
        let connection = match central::connect(softdevice, &connect_config).await {
            Ok(v) => v,
            #[allow(unused_variables)]
            Err(e) => {
                #[cfg(feature = "defmt")]
                error!("Failed to connect: {}", e);
                continue;
            }
        };
        info!("Connected to peripheral");

        debug!("Discovering BLE service...");
        let client: Client = match gatt_client::discover(&connection).await {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "defmt")]
                error!("Failed to discover service: {}", e);
                continue;
            }
        };
        info!("Discovered BLE service");

        if let Err(e) = client.message_to_central_cccd_write(true).await {
            #[cfg(feature = "defmt")]
            error!("Failed to set message_to_central_cccd_write: {}", e);
        }

        let recv = gatt_client::run(&connection, &client, |event| match event {
            ClientEvent::MessageToCentralNotification(message) => RECV_CHANNEL.send(message),
        });

        let send = async {
            loop {
                let message = SEND_CHANNEL.receive().await;
                #[allow(unused_variables)]
                if let Err(e) = client.message_to_peripheral_write(&message).await {
                    #[cfg(feature = "defmt")]
                    error!("Failed to write BLE message: {}", e);
                }
            }
        };

        select(recv, send).await;

        #[cfg(feature = "defmt")]
        warn!("GATT client disconnected");
    }
}

#[embassy_executor::task]
async fn peripheral(
    softdevice: &'static Softdevice,
    server: Server,
    address: [u8; 6],
    central_address: [u8; 6],
) {
    if !BLE_ADDRESS_WAS_SET.load(Ordering::SeqCst) {
        nrf_softdevice::ble::set_address(
            softdevice,
            &Address::new(AddressType::RandomStatic, address),
        );
        BLE_ADDRESS_WAS_SET.store(true, Ordering::SeqCst);
    }

    let adv = peripheral::ConnectableAdvertisement::NonscannableDirected {
        peer: Address::new(AddressType::RandomStatic, central_address),
    };
    let config = peripheral::Config::default();

    loop {
        debug!("Starting BLE peripheral advertisement...");
        let connection = match peripheral::advertise_connectable(softdevice, adv, &config).await {
            Ok(v) => v,
            #[allow(unused_variables)]
            Err(e) => {
                #[cfg(feature = "defmt")]
                error!("Failed to advertise: {}", e);
                continue;
            }
        };
        info!("Found BLE peripheral connection");

        debug!("Setting BLE sys attrs...");
        #[allow(unused_variables)]
        if let Err(e) = gatt_server::set_sys_attrs(&connection, None) {
            #[cfg(feature = "defmt")]
            error!("Failed to set sys attrs: {}", e);
            continue;
        }
        info!("Set BLE sys attrs");

        let recv = gatt_server::run(&connection, &server, |event| match event {
            ServerEvent::Service(event) => match event {
                ServiceEvent::MessageToPeripheralWrite(message) => RECV_CHANNEL.send(message),
                ServiceEvent::MessageToCentralCccdWrite { notifications: _ } => {}
            },
        });

        let send = async {
            loop {
                let message = SEND_CHANNEL.receive().await;
                #[allow(unused_variables)]
                if let Err(e) = server
                    .service
                    .message_to_central_notify(&connection, &message)
                {
                    #[cfg(feature = "defmt")]
                    error!("Failed to notify BLE message: {}", e)
                }
            }
        };

        select(recv, send).await;

        #[cfg(feature = "defmt")]
        warn!("GATT server disconnected");
    }
}
