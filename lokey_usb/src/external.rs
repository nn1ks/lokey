mod message_service;
mod transport;

use core::sync::atomic::Ordering;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_usb::driver::Driver;
use lokey::external::{self, NoMessage};
use lokey::util::{debug, info};
pub use message_service::{InitMessageService, RxMessageService, TxMessageService};
use portable_atomic::AtomicBool;
pub use transport::Transport;

#[derive(Clone)]
pub struct TransportConfig {
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: Option<&'static str>,
    pub product: Option<&'static str>,
    pub serial_number: Option<&'static str>,
    pub self_powered: bool,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            vendor_id: 0x1d51,
            product_id: 0x615f,
            manufacturer: None,
            product: None,
            serial_number: None,
            self_powered: false,
        }
    }
}

impl From<TransportConfig> for embassy_usb::Config<'static> {
    fn from(value: TransportConfig) -> Self {
        let mut config = embassy_usb::Config::new(value.vendor_id, value.product_id);
        config.manufacturer = value.manufacturer;
        config.product = value.product;
        config.serial_number = value.serial_number;
        config.self_powered = value.self_powered;
        config
    }
}

struct DeviceHandlerContext {
    configured: bool,
    suspended: AtomicBool,
    activation_request_signal: Signal<CriticalSectionRawMutex, ()>,
}

impl DeviceHandlerContext {
    pub fn new() -> Self {
        Self {
            configured: false,
            suspended: AtomicBool::new(false),
            activation_request_signal: Signal::new(),
        }
    }

    fn create_device_handler(&self) -> DeviceHandler<'_> {
        DeviceHandler {
            configured: self.configured,
            suspended: &self.suspended,
            activation_request_signal: &self.activation_request_signal,
        }
    }
}

struct DeviceHandler<'a> {
    configured: bool,
    suspended: &'a AtomicBool,
    activation_request_signal: &'a Signal<CriticalSectionRawMutex, ()>,
}

impl<'a> DeviceHandler<'a> {}

impl<'a> embassy_usb::Handler for DeviceHandler<'a> {
    fn enabled(&mut self, enabled: bool) {
        self.configured = false;
        self.suspended.store(false, Ordering::Release);
        #[allow(clippy::if_same_then_else)]
        if enabled {
            info!("USB device enabled");
        } else {
            info!("USB device disabled");
        }
    }

    fn reset(&mut self) {
        self.configured = false;
        debug!("Bus reset, the Vbus current limit is 100mA");
    }

    fn addressed(&mut self, addr: u8) {
        self.configured = false;
        info!("USB address set to: {}", addr);
        self.activation_request_signal.signal(());
    }

    fn configured(&mut self, configured: bool) {
        self.configured = configured;
        #[allow(clippy::if_same_then_else)]
        if configured {
            debug!(
                "USB device configured, it may now draw up to the configured current limit from Vbus."
            );
        } else {
            debug!("USB device is no longer configured, the Vbus current limit is 100mA.");
        }
    }

    fn suspended(&mut self, suspended: bool) {
        if suspended {
            debug!(
                "USB device suspended, the Vbus current limit is 500µA (or 2.5mA for high-power devices with remote wakeup enabled)."
            );
            self.suspended.store(true, Ordering::Release);
        } else {
            self.suspended.store(false, Ordering::Release);
            #[allow(clippy::if_same_then_else)]
            if self.configured {
                debug!(
                    "USB device resumed, it may now draw up to the configured current limit from Vbus"
                );
            } else {
                debug!("USB device resumed, the Vbus current limit is 100mA");
            }
        }
    }
}

pub trait TxMessage: external::Message + Sized {
    type MessageService<'d, D: Driver<'d>>: TxMessageService<Self> + InitMessageService<'d, D>;
}

pub trait RxMessage: external::Message + Sized {
    type MessageService<'d, D: Driver<'d>>: RxMessageService<Self> + InitMessageService<'d, D>;
}

impl TxMessage for NoMessage {
    type MessageService<'d, D: Driver<'d>> = ();
}

impl RxMessage for NoMessage {
    type MessageService<'d, D: Driver<'d>> = ();
}
