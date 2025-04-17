use super::{Messages, Transport};
use crate::mcu::Mcu;
use crate::util::channel::Channel;
use crate::util::{debug, error, info, unwrap};
use alloc::boxed::Box;
use core::marker::PhantomData;
use core::pin::Pin;
use core::sync::atomic::Ordering;
use embassy_executor::Spawner;
use embassy_executor::raw::TaskStorage;
use embassy_futures::join::join;
use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_usb::class::hid::HidWriter;
use portable_atomic::AtomicBool;
use portable_atomic_util::Arc;
use usbd_hid::descriptor::{AsInputReport, SerializedDescriptor};

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

pub struct HidTransport<const REPORT_SIZE: usize, M, T, R> {
    channel: Arc<Channel<CriticalSectionRawMutex, T>>,
    activation_request_signal: Arc<Signal<CriticalSectionRawMutex, ()>>,
    phantom: PhantomData<(M, R)>,
}

impl<const REPORT_SIZE: usize, M, T, R> HidTransport<REPORT_SIZE, M, T, R>
where
    M: Mcu + CreateDriver,
    T: Messages,
    R: AsInputReport + SerializedDescriptor + 'static,
{
    pub fn init<F>(
        config: TransportConfig,
        mcu: &'static M,
        spawner: Spawner,
        handle_message: F,
    ) -> Self
    where
        F: FnMut(T) -> Option<R> + 'static,
    {
        let channel = Arc::new(Channel::new());

        let device_handler = DeviceHandler::new();
        let activation_request_signal = Arc::clone(device_handler.activation_request_signal());

        async fn run<
            const REPORT_SIZE: usize,
            M: Mcu + CreateDriver,
            T: Messages,
            R: AsInputReport + SerializedDescriptor,
            F: FnMut(T) -> Option<R>,
        >(
            config: TransportConfig,
            mcu: &'static M,
            channel: Arc<Channel<CriticalSectionRawMutex, T>>,
            mut device_handler: DeviceHandler,
            mut handle_message: F,
        ) {
            let driver = mcu.create_driver();

            let mut config = embassy_usb::Config::from(config);
            config.supports_remote_wakeup = true;

            let mut config_descriptor = [0; 256];
            let mut bos_descriptor = [0; 256];
            let mut msos_descriptor = [0; 256];
            let mut control_buf = [0; 64];

            let mut state = embassy_usb::class::hid::State::new();

            let suspended = Arc::clone(device_handler.suspended());

            let mut builder = embassy_usb::Builder::new(
                driver,
                config,
                &mut config_descriptor,
                &mut bos_descriptor,
                &mut msos_descriptor,
                &mut control_buf,
            );

            builder.handler(&mut device_handler);

            let hid_config = embassy_usb::class::hid::Config {
                report_descriptor: R::desc(),
                request_handler: None,
                poll_ms: 60,
                max_packet_size: 64,
            };
            let mut hid_writer =
                HidWriter::<_, REPORT_SIZE>::new(&mut builder, &mut state, hid_config);

            let mut usb = builder.build();

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
                loop {
                    let message = channel.receive().await;
                    if suspended.load(Ordering::Acquire) {
                        info!("Triggering remote wakeup");
                        remote_wakeup.signal(());
                    } else if let Some(report) = handle_message(message) {
                        if let Err(e) = hid_writer.write_serialize(&report).await {
                            error!("Failed to write input report: {}", e);
                        }
                    }
                }
            };

            join(wakeup, write_keyboard_report).await.0
        }

        let channel_clone = Arc::clone(&channel);
        let task_storage = Box::leak(Box::new(TaskStorage::new()));
        let task = task_storage.spawn(|| {
            run::<REPORT_SIZE, _, _, _, _>(
                config,
                mcu,
                channel_clone,
                device_handler,
                handle_message,
            )
        });
        unwrap!(spawner.spawn(task));

        Self {
            channel,
            activation_request_signal,
            phantom: PhantomData,
        }
    }
}

impl<const REPORT_SIZE: usize, M, T, R> Transport for HidTransport<REPORT_SIZE, M, T, R>
where
    M: Mcu,
    T: Messages,
    R: 'static,
{
    type Messages = T;

    fn send(&self, message: Self::Messages) {
        self.channel.send(message);
    }

    fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(self.activation_request_signal.wait())
    }
}
