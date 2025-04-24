mod hid_transport;

use crate::mcu::Mcu;
use crate::util::{debug, info};
use core::sync::atomic::Ordering;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
pub use hid_transport::HidTransport;
use portable_atomic::AtomicBool;
use portable_atomic_util::Arc;

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

pub trait CreateDriver: Mcu {
    fn create_driver<'a>(&'static self) -> impl embassy_usb::driver::Driver<'a>;
}

pub struct DeviceHandler {
    configured: bool,
    suspended: Arc<AtomicBool>,
    activation_request_signal: Arc<Signal<CriticalSectionRawMutex, ()>>,
}

impl DeviceHandler {
    pub fn new() -> Self {
        Self {
            configured: false,
            suspended: Arc::new(AtomicBool::new(false)),
            activation_request_signal: Arc::new(Signal::new()),
        }
    }

    pub fn suspended(&self) -> &Arc<AtomicBool> {
        &self.suspended
    }

    pub fn activation_request_signal(&self) -> &Arc<Signal<CriticalSectionRawMutex, ()>> {
        &self.activation_request_signal
    }
}

impl Default for DeviceHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl embassy_usb::Handler for DeviceHandler {
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
