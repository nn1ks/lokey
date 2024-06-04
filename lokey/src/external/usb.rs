use crate::{external, mcu::Mcu};
use core::pin::pin;
use core::sync::atomic::Ordering;
use defmt::{debug, info, unwrap, warn};
use embassy_futures::join::join;
use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_usb::class::hid::{HidReaderWriter, ReportId, State};
use embassy_usb::control::OutResponse;
use futures_util::{Stream, StreamExt};
use portable_atomic::AtomicBool;
use portable_atomic_util::Arc;
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};

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

pub struct ActivationRequest {
    signal: Arc<Signal<CriticalSectionRawMutex, ()>>,
}

impl ActivationRequest {
    pub async fn wait(&self) {
        self.signal.wait().await;
    }
}

pub struct Handler<M: 'static> {
    config: TransportConfig,
    mcu: &'static M,
    activation_request_signal: Arc<Signal<CriticalSectionRawMutex, ()>>,
}

impl<M: CreateDriver> Handler<M> {
    pub fn new(config: TransportConfig, mcu: &'static M) -> (Self, ActivationRequest) {
        let signal = Arc::new(Signal::new());
        let handler = Self {
            config,
            mcu,
            activation_request_signal: Arc::clone(&signal),
        };
        let activation_request = ActivationRequest { signal };
        (handler, activation_request)
    }

    pub async fn run<S: Stream<Item = external::Message>>(self, message_stream: S) -> ! {
        let driver = self.mcu.create_driver();

        let mut config = embassy_usb::Config::from(self.config);
        config.max_power = 100;
        config.max_packet_size_0 = 64;
        config.supports_remote_wakeup = true;

        let suspended = Arc::new(AtomicBool::new(false));

        let mut device_descriptor = [0; 256];
        let mut config_descriptor = [0; 256];
        let mut bos_descriptor = [0; 256];
        let mut msos_descriptor = [0; 256];
        let mut control_buf = [0; 64];
        let mut device_handler = DeviceHandler {
            configured: false,
            suspended: Arc::clone(&suspended),
            activation_request_signal: self.activation_request_signal,
        };

        let mut state = State::new();

        let mut builder = embassy_usb::Builder::new(
            driver,
            config,
            &mut device_descriptor,
            &mut config_descriptor,
            &mut bos_descriptor,
            &mut msos_descriptor,
            &mut control_buf,
        );

        builder.handler(&mut device_handler);

        let config = embassy_usb::class::hid::Config {
            report_descriptor: KeyboardReport::desc(),
            request_handler: None,
            poll_ms: 60,
            max_packet_size: 64,
        };
        let hid = HidReaderWriter::<_, 1, 8>::new(&mut builder, &mut state, config);

        let (reader, mut writer) = hid.split();

        let mut usb = builder.build();

        let request_handler = RequestHandler {};

        let remote_wakeup: Signal<CriticalSectionRawMutex, ()> = Signal::new();

        let wakeup = async {
            loop {
                usb.run_until_suspend().await;
                match select(usb.wait_resume(), remote_wakeup.wait()).await {
                    Either::First(()) => {
                        suspended.store(false, Ordering::Release);
                    }
                    Either::Second(()) => {
                        unwrap!(usb.remote_wakeup().await);
                    }
                }
            }
        };

        let write_keyboard_report = async {
            // FIXME: If multiple keys with the same keycode are pressed and then one key is released, the keycode will not be sent anymore.
            let mut report = KeyboardReport {
                modifier: 0,
                reserved: 0,
                leds: 0,
                keycodes: [0; 6],
            };
            let mut message_stream = pin!(message_stream);
            loop {
                if let Some(message) = message_stream.next().await {
                    if suspended.load(Ordering::Acquire) {
                        info!("Triggering remote wakeup");
                        remote_wakeup.signal(());
                    } else {
                        let report_changed = message.update_keyboard_report(&mut report);
                        if report_changed {
                            match writer.write_serialize(&report).await {
                                Ok(()) => {}
                                Err(e) => warn!("Failed to send report: {:?}", e),
                            };
                        }
                    }
                }
            }
        };

        let handle_requests = async { reader.run(false, &request_handler).await };

        join(wakeup, join(write_keyboard_report, handle_requests))
            .await
            .0
    }
}

struct RequestHandler {}

impl embassy_usb::class::hid::RequestHandler for RequestHandler {
    fn get_report(&self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        debug!("Get report for {:?}", id);
        None
    }

    fn set_report(&self, id: ReportId, data: &[u8]) -> OutResponse {
        debug!("Set report for {:?}: {=[u8]}", id, data);
        OutResponse::Accepted
    }

    fn set_idle_ms(&self, id: Option<ReportId>, dur: u32) {
        debug!("Set idle rate for {:?} to {:?}", id, dur);
    }

    fn get_idle_ms(&self, id: Option<ReportId>) -> Option<u32> {
        debug!("Get idle rate for {:?}", id);
        None
    }
}

struct DeviceHandler {
    configured: bool,
    suspended: Arc<AtomicBool>,
    activation_request_signal: Arc<Signal<CriticalSectionRawMutex, ()>>,
}

impl embassy_usb::Handler for DeviceHandler {
    fn enabled(&mut self, enabled: bool) {
        self.configured = false;
        self.suspended.store(false, Ordering::Release);
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
            debug!("USB device suspended, the Vbus current limit is 500µA (or 2.5mA for high-power devices with remote wakeup enabled).");
            self.suspended.store(true, Ordering::Release);
        } else {
            self.suspended.store(false, Ordering::Release);
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
