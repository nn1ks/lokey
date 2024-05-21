use crate::{external, mcu::Mcu};
use core::sync::atomic::{AtomicBool, Ordering};
use defmt::{info, unwrap, warn};
use embassy_futures::join::join;
use embassy_futures::select::{select, Either};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Receiver, signal::Signal,
};
use embassy_usb::class::hid::{HidReaderWriter, ReportId, State};
use embassy_usb::{control::OutResponse, Handler};
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};

pub struct ChannelConfig {
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: Option<&'static str>,
    pub product: Option<&'static str>,
    pub serial_number: Option<&'static str>,
    pub self_powered: bool,
}

impl Default for ChannelConfig {
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

impl From<ChannelConfig> for embassy_usb::Config<'static> {
    fn from(value: ChannelConfig) -> Self {
        let mut config = embassy_usb::Config::new(value.vendor_id, value.product_id);
        config.manufacturer = value.manufacturer;
        config.product = value.product;
        config.serial_number = value.serial_number;
        config.self_powered = value.self_powered;
        config
    }
}

static SUSPENDED: AtomicBool = AtomicBool::new(false);
pub static ACTIVATION_REQUEST: Signal<CriticalSectionRawMutex, ()> = Signal::new();

pub trait Driver: Mcu {
    fn driver<'a>(&'static self) -> impl embassy_usb::driver::Driver<'a>;
}

pub async fn common<M: Driver, const N: usize>(
    config: ChannelConfig,
    mcu: &'static M,
    receiver: Receiver<'_, CriticalSectionRawMutex, external::Message, N>,
) {
    let driver = mcu.driver();

    let mut config = embassy_usb::Config::from(config);
    config.max_power = 100;
    config.max_packet_size_0 = 64;
    config.supports_remote_wakeup = true;

    let mut device_descriptor = [0; 256];
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; 64];
    let mut device_handler = DeviceHandler {
        configured: AtomicBool::new(false),
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

    let usb_fut = async {
        loop {
            usb.run_until_suspend().await;
            match select(usb.wait_resume(), remote_wakeup.wait()).await {
                Either::First(_) => (),
                Either::Second(_) => unwrap!(usb.remote_wakeup().await),
            }
        }
    };

    let in_fut = async {
        // FIXME: If multiple keys with the same keycode are pressed and then one key is released, the keycode will not be sent anymore.
        let mut report = KeyboardReport {
            modifier: 0,
            reserved: 0,
            leds: 0,
            keycodes: [0; 6],
        };
        loop {
            let message = receiver.receive().await;
            let report_changed = message.update_keyboard_report(&mut report);
            if report_changed {
                match writer.write_serialize(&report).await {
                    Ok(()) => {}
                    Err(e) => warn!("Failed to send report: {:?}", e),
                };
            }
        }
    };

    let out_fut = async {
        reader.run(false, &request_handler).await;
    };

    join(usb_fut, join(in_fut, out_fut)).await;
}

struct RequestHandler {}

impl embassy_usb::class::hid::RequestHandler for RequestHandler {
    fn get_report(&self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        info!("Get report for {:?}", id);
        None
    }

    fn set_report(&self, id: ReportId, data: &[u8]) -> OutResponse {
        info!("Set report for {:?}: {=[u8]}", id, data);
        OutResponse::Accepted
    }

    fn set_idle_ms(&self, id: Option<ReportId>, dur: u32) {
        info!("Set idle rate for {:?} to {:?}", id, dur);
    }

    fn get_idle_ms(&self, id: Option<ReportId>) -> Option<u32> {
        info!("Get idle rate for {:?}", id);
        None
    }
}

struct DeviceHandler {
    configured: AtomicBool,
}

impl Handler for DeviceHandler {
    fn enabled(&mut self, enabled: bool) {
        self.configured.store(false, Ordering::Relaxed);
        SUSPENDED.store(false, Ordering::Release);
        #[allow(clippy::if_same_then_else)] // seems to be a bug in clippy
        if enabled {
            info!("Device enabled");
        } else {
            info!("Device disabled");
        }
    }

    fn reset(&mut self) {
        self.configured.store(false, Ordering::Relaxed);
        info!("Bus reset, the Vbus current limit is 100mA");
    }

    fn addressed(&mut self, addr: u8) {
        self.configured.store(false, Ordering::Relaxed);
        info!("USB address set to: {}", addr);
        ACTIVATION_REQUEST.signal(());
    }

    fn configured(&mut self, configured: bool) {
        self.configured.store(configured, Ordering::Relaxed);
        #[allow(clippy::if_same_then_else)] // seems to be a bug in clippy
        if configured {
            info!(
                "Device configured, it may now draw up to the configured current limit from Vbus."
            )
        } else {
            info!("Device is no longer configured, the Vbus current limit is 100mA.");
        }
    }

    fn suspended(&mut self, suspended: bool) {
        if suspended {
            info!("Device suspended, the Vbus current limit is 500µA (or 2.5mA for high-power devices with remote wakeup enabled).");
            SUSPENDED.store(true, Ordering::Release);
        } else {
            SUSPENDED.store(false, Ordering::Release);
            #[allow(clippy::if_same_then_else)] // seems to be a bug in clippy
            if self.configured.load(Ordering::Relaxed) {
                info!(
                    "Device resumed, it may now draw up to the configured current limit from Vbus"
                );
            } else {
                info!("Device resumed, the Vbus current limit is 100mA");
            }
        }
    }
}
