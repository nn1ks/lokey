mod bonder;
mod server;

use super::{BLE_ADDRESS_WAS_SET, device_address_to_ble_address};
use crate::external::Messages1;
use crate::external::ble::{Event, Message};
use crate::mcu::{Nrf52840, Storage};
use crate::util::channel::Channel;
use crate::util::{debug, error, info, unwrap, warn};
use crate::{Address, external, internal, keyboard};
use alloc::boxed::Box;
use alloc::vec::Vec;
use bonder::Bonder;
use core::num::NonZeroU8;
use core::sync::atomic::Ordering;
use embassy_executor::Spawner;
use embassy_futures::join::join4;
use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use nrf_softdevice::ble::advertisement_builder::{
    AdvertisementDataType, Flag, LegacyAdvertisementBuilder, LegacyAdvertisementPayload,
    ServiceList, ServiceUuid16,
};
use nrf_softdevice::ble::security::SecurityHandler;
use nrf_softdevice::ble::{gatt_server, peripheral};
use nrf_softdevice::{Flash, Softdevice};
use portable_atomic::{AtomicBool, AtomicU8};
use server::{BatteryServiceEvent, HidServiceEvent, Server, ServerEvent};
use usbd_hid::descriptor::KeyboardReport;

static CHANNEL: Channel<CriticalSectionRawMutex, keyboard::ExternalMessage> = Channel::new();
static ACTIVE_SIGNAL: Signal<CriticalSectionRawMutex, bool> = Signal::new();
static IS_ACTIVE: AtomicBool = AtomicBool::new(true);

#[non_exhaustive]
pub struct Transport {}

impl external::Transport for Transport {
    type Messages = Messages1<keyboard::ExternalMessage>;

    fn send(&self, message: Messages1<keyboard::ExternalMessage>) {
        let Messages1::Message1(message) = message;
        CHANNEL.send(message);
    }

    fn set_active(&self, value: bool) -> bool {
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

impl external::TransportConfig<Nrf52840, Messages1<keyboard::ExternalMessage>>
    for external::ble::TransportConfig
{
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

        let Some(num_profiles) = NonZeroU8::new(self.num_profiles) else {
            return Transport {};
        };

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
            num_profiles,
            internal_channel,
            spawner
        )));
        INITIALIZED.store(true, Ordering::SeqCst);
        Transport {}
    }
}

#[allow(clippy::too_many_arguments)]
#[embassy_executor::task]
async fn task(
    server: Server,
    softdevice: &'static Softdevice,
    storage: &'static Storage<Flash>,
    name: &'static str,
    address: Address,
    num_profiles: NonZeroU8,
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

    let config = peripheral::Config::default();

    let mut bond_infos = Vec::new();
    for i in 0..num_profiles.get() {
        match storage.fetch::<bonder::BondInfo>(i).await {
            Ok(v) => bond_infos.push(v),
            Err(e) => error!("Failed to read bond info from flash: {}", e),
        }
    }

    unwrap!(spawner.spawn(bonder::handle_messages(storage)));
    let bonder = Box::leak(Box::new(Bonder::new(bond_infos, AtomicU8::new(0))));

    if !BLE_ADDRESS_WAS_SET.load(Ordering::SeqCst) {
        nrf_softdevice::ble::set_address(softdevice, &device_address_to_ble_address(&address));
        BLE_ADDRESS_WAS_SET.store(true, Ordering::SeqCst);
    }

    let cancel_activation_wait = Signal::<CriticalSectionRawMutex, ()>::new();
    let cancel_advertisement = Signal::<CriticalSectionRawMutex, ()>::new();

    let connection = Mutex::<CriticalSectionRawMutex, _>::new(None);
    let active_profile_index: AtomicU8 = AtomicU8::new(0);
    let run_ble_server = async {
        loop {
            while !IS_ACTIVE.load(Ordering::Acquire) {
                cancel_activation_wait.wait().await;
                cancel_advertisement.reset();
            }

            let profile_index = active_profile_index.load(Ordering::SeqCst);
            let found_bond_info = bonder.bond_infos.borrow()[profile_index as usize].is_some();
            bonder
                .active_profile_index
                .store(profile_index, Ordering::SeqCst);
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

            internal_channel.send(Event::StartedAdvertising {
                scannable: !found_bond_info,
            });
            let advertise = peripheral::advertise_pairable(softdevice, adv, &config, bonder);
            let new_connection = match select(advertise, cancel_advertisement.wait()).await {
                Either::First(v) => v,
                Either::Second(()) => {
                    info!("Cancelled BLE advertisement");
                    internal_channel.send(Event::StoppedAdvertising {
                        scannable: !found_bond_info,
                    });
                    continue;
                }
            };

            let new_connection = match new_connection {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to advertise: {}", e);
                    internal_channel.send(Event::StoppedAdvertising {
                        scannable: !found_bond_info,
                    });
                    continue;
                }
            };
            bonder.load_sys_attrs(&new_connection);
            *connection.lock().await = Some(new_connection.clone());

            internal_channel.send(Event::StoppedAdvertising {
                scannable: !found_bond_info,
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
            cancel_advertisement.reset();

            warn!("GATT server disconnected");
            *connection.lock().await = None;
            internal_channel.send(Event::Disconnected { device_address });

            let bond_info = match storage.fetch::<bonder::BondInfo>(profile_index).await {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to read bond info from flash: {}", e);
                    None
                }
            };
            *bonder
                .bond_infos
                .borrow_mut()
                .get_mut(profile_index as usize)
                .unwrap() = bond_info;
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
                            if let Some(connection) = &mut *connection.lock().await {
                                let _ = connection.disconnect();
                            }
                            cancel_advertisement.signal(());
                        }
                        internal_channel.send(Event::SwitchedProfile {
                            profile_index: index,
                            changed: is_different_profile,
                        });
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
                    if let Some(connection) = &mut *connection.lock().await {
                        let _ = connection.disconnect();
                    }
                    cancel_advertisement.signal(());
                    internal_channel.send(Event::SwitchedProfile {
                        profile_index: new_profile_index,
                        changed: num_profiles.get() > 1,
                    });
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
                    if let Some(connection) = &mut *connection.lock().await {
                        let _ = connection.disconnect();
                    }
                    cancel_advertisement.signal(());
                    internal_channel.send(Event::SwitchedProfile {
                        profile_index: new_profile_index,
                        changed: num_profiles.get() > 1,
                    });
                }
                Message::DisconnectActive => {
                    if let Some(connection) = &mut *connection.lock().await {
                        let _ = connection.disconnect();
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
                        if let Err(e) = storage.remove::<bonder::BondInfo>(profile_index).await {
                            error!("Failed to remove bond info: {}", e);
                        }
                        bonder.set_bond_info(profile_index, None);
                        if active_profile_index.load(Ordering::SeqCst) == profile_index {
                            if let Some(connection) = &mut *connection.lock().await {
                                let _ = connection.disconnect();
                            }
                            cancel_advertisement.signal(());
                        }
                    }
                }
                Message::ClearActive => {
                    debug!("Removing bond info for active profile");
                    let profile_index = active_profile_index.load(Ordering::SeqCst);
                    if let Err(e) = storage.remove::<bonder::BondInfo>(profile_index).await {
                        error!("Failed to remove bond info: {}", e);
                    }
                    bonder.set_bond_info(profile_index, None);
                    if let Some(connection) = &mut *connection.lock().await {
                        let _ = connection.disconnect();
                    }
                    cancel_advertisement.signal(());
                }
                Message::ClearAll => {
                    debug!("Removing all bond infos");
                    for i in 0..num_profiles.get() {
                        if let Err(e) = storage.remove::<bonder::BondInfo>(i).await {
                            error!("Failed to remove bond info: {}", e);
                        }
                        bonder.set_bond_info(i, None);
                    }
                    if let Some(connection) = &mut *connection.lock().await {
                        let _ = connection.disconnect();
                    }
                    cancel_advertisement.signal(());
                }
            }
        }
    };
    let handle_activation = async {
        loop {
            ACTIVE_SIGNAL.wait().await;
            if let Some(connection) = &mut *connection.lock().await {
                let _ = connection.disconnect();
            }
            cancel_activation_wait.signal(());
            cancel_advertisement.signal(());
        }
    };
    join4(
        run_ble_server,
        send_keyboard_report,
        handle_internal_messages,
        handle_activation,
    )
    .await;
}
