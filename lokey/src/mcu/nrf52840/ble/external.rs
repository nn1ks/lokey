mod bonder;
mod server;

use super::{BLE_ADDRESS_WAS_SET, device_address_to_ble_address};
use crate::external::ble::{Event, Message};
use crate::external::{self};
use crate::mcu::{Nrf52840, Storage};
use crate::util::channel::Channel;
use crate::util::{debug, error, info, unwrap, warn};
use crate::{Address, internal};
use alloc::boxed::Box;
use bonder::Bonder;
use core::sync::atomic::Ordering;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use nrf_softdevice::ble::advertisement_builder::{
    AdvertisementDataType, Flag, LegacyAdvertisementBuilder, LegacyAdvertisementPayload,
    ServiceList, ServiceUuid16,
};
use nrf_softdevice::ble::{gatt_server, peripheral};
use nrf_softdevice::{Flash, Softdevice};
use portable_atomic::AtomicBool;
use server::{BatteryServiceEvent, HidServiceEvent, Server, ServerEvent};
use usbd_hid::descriptor::KeyboardReport;

static CHANNEL: Channel<CriticalSectionRawMutex, external::Message> = Channel::new();

#[non_exhaustive]
pub struct Transport {}

impl external::Transport for Transport {
    fn send(&self, message: external::Message) {
        CHANNEL.send(message);
    }
}

impl external::TransportConfig<Nrf52840> for external::ble::TransportConfig {
    type Transport = Transport;
    async fn init(
        self,
        mcu: &'static Nrf52840,
        address: Address,
        spawner: Spawner,
        internal_channel: internal::DynChannel,
    ) -> Self::Transport {
        static INITIALIZED: AtomicBool = AtomicBool::new(false);

        if INITIALIZED.load(Ordering::SeqCst) {
            return Transport {};
        }

        let name = self.name;
        let softdevice: &'static mut Softdevice = unsafe { &mut *mcu.softdevice.get() };
        let server = unwrap!(Server::new(softdevice, &self));
        let softdevice: &'static Softdevice = softdevice;
        unwrap!(spawner.spawn(task(
            server,
            softdevice,
            mcu.storage,
            name,
            address,
            internal_channel,
            spawner
        )));
        INITIALIZED.store(true, Ordering::SeqCst);
        Transport {}
    }
}

#[embassy_executor::task]
async fn task(
    server: Server,
    softdevice: &'static Softdevice,
    storage: &'static Storage<Flash>,
    name: &'static str,
    address: Address,
    internal_channel: internal::DynChannel,
    spawner: Spawner,
) {
    let adv_data: LegacyAdvertisementPayload = LegacyAdvertisementBuilder::new()
        .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
        .services_16(
            ServiceList::Incomplete,
            &[
                ServiceUuid16::BATTERY,
                ServiceUuid16::HUMAN_INTERFACE_DEVICE,
            ],
        )
        .full_name(name)
        // Change the appearance (icon of the bluetooth device) to a keyboard
        .raw(AdvertisementDataType::APPEARANCE, &[0xC1, 0x03])
        .build();

    let scan_data: LegacyAdvertisementPayload = LegacyAdvertisementBuilder::new()
        .services_16(
            ServiceList::Complete,
            &[
                ServiceUuid16::DEVICE_INFORMATION,
                ServiceUuid16::BATTERY,
                ServiceUuid16::HUMAN_INTERFACE_DEVICE,
            ],
        )
        .build();

    let bond_info = match storage.fetch::<bonder::BondInfo>(0).await {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to read bond info from flash: {}", e);
            None
        }
    };

    unwrap!(spawner.spawn(bonder::handle_messages(storage)));
    let bonder = Box::leak(Box::new(Bonder::new(bond_info)));

    if !BLE_ADDRESS_WAS_SET.load(Ordering::SeqCst) {
        nrf_softdevice::ble::set_address(softdevice, &device_address_to_ble_address(&address));
        BLE_ADDRESS_WAS_SET.store(true, Ordering::SeqCst);
    }

    let connection = Mutex::<CriticalSectionRawMutex, _>::new(None);
    let run_ble_server = async {
        let config = peripheral::Config::default();
        loop {
            let found_bond_info = bonder.bond_info.borrow().is_some();
            internal_channel.send(Event::StartedAdvertising {
                scannable: !found_bond_info,
            });
            let adv = if found_bond_info {
                info!("Advertising as non-scannable because of existing bond info");
                peripheral::ConnectableAdvertisement::ExtendedNonscannableUndirected {
                    set_id: 0,
                    adv_data: &adv_data,
                }
            } else {
                info!("Advertising as scannable because of no stored bond info");
                peripheral::ConnectableAdvertisement::ScannableUndirected {
                    adv_data: &adv_data,
                    scan_data: &scan_data,
                }
            };
            let new_connection =
                match peripheral::advertise_pairable(softdevice, adv, &config, bonder).await {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Failed to advertise: {}", e);
                        continue;
                    }
                };
            *connection.lock().await = Some(new_connection.clone());
            internal_channel.send(Event::StoppedAdvertising {
                scannable: found_bond_info,
            });
            let device_address = Address(new_connection.peer_address().bytes());
            internal_channel.send(Event::Connected { device_address });

            info!("Advertising done, found connection");

            // Run the GATT server on the connection. This returns when the connection gets disconnected.
            gatt_server::run(&new_connection, &server, |event| match event {
                ServerEvent::Battery(e) => match e {
                    BatteryServiceEvent::BatteryLevelCccdWrite { notifications } => {
                        debug!(
                            "Received event BatteryLevelCcdWrite {{ notifications: {} }}",
                            notifications
                        )
                    }
                },
                ServerEvent::Hid(e) => match e {
                    HidServiceEvent::InputReportWrite(v) => {
                        debug!("Received event InputReportWrite({})", v)
                    }
                    HidServiceEvent::InputReportCccdWrite { notifications } => {
                        debug!(
                            "Received event InputReportCcdWrite {{ notifications: {} }}",
                            notifications
                        )
                    }
                },
            })
            .await;

            internal_channel.send(Event::Disconnected { device_address });
            warn!("GATT server disconnected");

            let bond_info = match storage.fetch::<bonder::BondInfo>(0).await {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to read bond info from flash: {}", e);
                    None
                }
            };
            *bonder.bond_info.borrow_mut() = bond_info;
        }
    };
    let send_keyboard_report = async {
        let mut report = KeyboardReport {
            modifier: 0,
            reserved: 0,
            leds: 0,
            keycodes: [0; 6],
        };
        loop {
            let message = CHANNEL.receive().await;
            match &*connection.lock().await {
                Some(conn) => {
                    let report_changed = message.update_keyboard_report(&mut report);
                    if report_changed {
                        let mut report_bytes = [0u8; 8];
                        ssmarshal::serialize(&mut report_bytes, &report).unwrap();
                        if let Err(e) = server.hid_service.input_report_notify(conn, &report_bytes)
                        {
                            error!("Failed to notify about keyboard report: {}", e);
                        }
                    }
                }
                None => {
                    warn!("Ignoring external message as there is no bluetooth connection");
                }
            }
        }
    };
    let handle_internal_messages = async {
        let mut receiver = internal_channel.receiver::<Message>();
        loop {
            let message = receiver.next().await;
            match message {
                Message::Disconnect => {
                    if let Some(connection) = &mut *connection.lock().await {
                        let _ = connection.disconnect();
                    }
                }
                Message::Clear => {
                    debug!("Removing bond info");
                    if let Err(e) = storage.remove::<bonder::BondInfo>(0).await {
                        error!("Failed to remove bond info: {}", e);
                    }
                    if let Some(connection) = &mut *connection.lock().await {
                        let _ = connection.disconnect();
                    }
                }
            }
        }
    };
    join(
        join(run_ble_server, send_keyboard_report),
        handle_internal_messages,
    )
    .await;
}
