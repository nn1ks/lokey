use crate::{internal, mcu::Nrf52840, util::channel::Channel};
use alloc::{boxed::Box, vec::Vec};
use core::{future::Future, pin::Pin};
use defmt::{debug, error, info, unwrap, warn};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use nrf_softdevice::ble::advertisement_builder::{Flag, LegacyAdvertisementBuilder, ServiceList};
use nrf_softdevice::ble::{
    central, gatt_client, gatt_server, peripheral, Address, AddressType, Connection,
};
use nrf_softdevice::Softdevice;
use nrf_softdevice::{ble::GattValue, gatt_client, gatt_server, gatt_service};

const PERIPHERAL_SERVICE_UUID: [u8; 16] = [
    0x68, 0x8f, 0x93, 0x20, 0x01, 0xe9, 0x22, 0xa8, 0x0f, 0x43, 0x28, 0xdd, 0x3d, 0xb1, 0xb4, 0xc6,
];
const CENTRAL_SERVICE_UUID: [u8; 16] = [
    0x85, 0xa9, 0x58, 0x3e, 0xa5, 0x70, 0x1b, 0x8b, 0xfe, 0x41, 0x9b, 0xd3, 0x5f, 0x03, 0x51, 0x2e,
];

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
    #[characteristic(uuid = "f732a64b-06bb-4fec-94ac-7abfe1119ac8", read, write, notify)]
    message: Message,
}

#[gatt_client(uuid = "c6b4b13d-dd28-430f-a822-e90120938f68")]
#[derive(Clone)]
struct Client {
    #[characteristic(uuid = "acbfbc81-99a0-4c71-be47-0bbde038fc4c", read, write, notify)]
    message: Message,
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
        if self.central {
            unwrap!(spawner.spawn(central(softdevice)));
        } else {
            let server = unwrap!(Server::new(softdevice));
            unwrap!(spawner.spawn(peripheral(softdevice, server)));
        }
        Transport {}
    }
}

#[embassy_executor::task]
async fn central(softdevice: &'static Softdevice) {
    let scan_config = central::ScanConfig::default();

    let client: Mutex<CriticalSectionRawMutex, Option<Client>> = Mutex::new(None);

    let recv = async {
        loop {
            let result = central::scan(softdevice, &scan_config, |params| {
                let mut data = unsafe {
                    core::slice::from_raw_parts(params.data.p_data, params.data.len as usize)
                };
                let address_type = match AddressType::try_from(params.peer_addr.addr_type()) {
                    Ok(AddressType::Anonymous) => {
                        debug!("Ignoring advertisment from anonymous peer");
                        return None;
                    }
                    Ok(v) => v,
                    Err(_) => {
                        warn!(
                            "Unknown peer address type \"{}\"",
                            params.peer_addr.addr_type()
                        );
                        return None;
                    }
                };
                while !data.is_empty() {
                    let len = data[0] as usize;
                    if data.len() < len + 1 || len < 1 {
                        warn!("Invalid advertisement data");
                        break;
                    }
                    let key = data[1];
                    let value = &data[2..len + 1];
                    if key == 0x06 || key == 0x07 {
                        if value.len() % 128 != 0 {
                            warn!("Invalid data length for list of 128-bit services");
                            break;
                        }
                        let mut services = value;
                        while !services.is_empty() {
                            let uuid = &services[..128];
                            if uuid == PERIPHERAL_SERVICE_UUID {
                                let address = Address::new(address_type, params.peer_addr.addr);
                                return Some(address);
                            }
                            services = &services[128..];
                        }
                    }
                    data = &data[len + 1..];
                }
                None
            })
            .await;

            let peer_address = match result {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to scan: {}", e);
                    continue;
                }
            };
            info!("Found peripheral peer with address {}", peer_address);

            let mut config = central::ConnectConfig::default();
            let peer_addresses = &[&peer_address];
            config.scan_config.whitelist = Some(peer_addresses);

            // TODO: Change to `connect_with_security`
            let new_connection = match central::connect(softdevice, &config).await {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to connect: {}", e);
                    continue;
                }
            };

            let new_client = match gatt_client::discover::<Client>(&new_connection).await {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to discover service: {}", e);
                    continue;
                }
            };

            if let Err(e) = new_client.message_cccd_write(true).await {
                error!("Failed to write BLE message: {}", e);
            }

            *client.lock().await = Some(new_client.clone());

            gatt_client::run(&new_connection, &new_client, |event| match event {
                ClientEvent::MessageNotification(message) => RECV_CHANNEL.send(message),
            })
            .await;

            warn!("GATT client disconnected");
        }
    };

    let send = async {
        loop {
            let message = SEND_CHANNEL.receive().await;
            if let Some(client) = &*client.lock().await {
                if let Err(e) = client.message_write(&message).await {
                    error!("Failed to write BLE message: {}", e);
                }
            }
        }
    };

    join(recv, send).await;
}

#[embassy_executor::task]
async fn peripheral(softdevice: &'static Softdevice, server: Server) {
    let adv_data = LegacyAdvertisementBuilder::new()
        .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
        .services_128(ServiceList::Complete, &[PERIPHERAL_SERVICE_UUID])
        .build();

    let scan_data = LegacyAdvertisementBuilder::new()
        .services_128(ServiceList::Complete, &[CENTRAL_SERVICE_UUID])
        .build();

    let adv = peripheral::ConnectableAdvertisement::ScannableUndirected {
        adv_data: &adv_data,
        scan_data: &scan_data,
    };
    let config = peripheral::Config::default();

    let connection: Mutex<CriticalSectionRawMutex, Option<Connection>> = Mutex::new(None);

    let recv = async {
        loop {
            // TODO: Change to `advertise_pairable`
            let new_connection =
                match peripheral::advertise_connectable(softdevice, adv, &config).await {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Failed to advertise: {}", e);
                        continue;
                    }
                };

            *connection.lock().await = Some(new_connection.clone());

            gatt_server::run(&new_connection, &server, |event| match event {
                ServerEvent::Service(event) => match event {
                    ServiceEvent::MessageWrite(message) => RECV_CHANNEL.send(message),
                    ServiceEvent::MessageCccdWrite { notifications: _ } => {}
                },
            })
            .await;

            warn!("GATT server disconnected");
        }
    };

    let send = async {
        loop {
            let message = SEND_CHANNEL.receive().await;
            if let Some(connection) = &*connection.lock().await {
                if let Err(e) = server.service.message_notify(connection, &message) {
                    error!("Failed to notify BLE message: {}", e)
                }
            }
        }
    };

    join(recv, send).await;
}
