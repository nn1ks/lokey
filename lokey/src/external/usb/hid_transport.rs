use super::{CreateDriver, DeviceHandler, TransportConfig};
use crate::util::channel::Channel;
use crate::util::{error, info};
use crate::{external, mcu};
use alloc::boxed::Box;
use core::marker::PhantomData;
use core::pin::Pin;
use core::sync::atomic::Ordering;
use embassy_futures::join::join;
use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_usb::class::hid::HidWriter;
use portable_atomic_util::Arc;
use usbd_hid::descriptor::{AsInputReport, SerializedDescriptor};

pub struct HidTransport<const REPORT_SIZE: usize, M: 'static, T, R> {
    channel: Channel<CriticalSectionRawMutex, T>,
    device_handler: Mutex<CriticalSectionRawMutex, DeviceHandler>,
    activation_request_signal: Arc<Signal<CriticalSectionRawMutex, ()>>,
    config: TransportConfig,
    mcu: &'static M,
    phantom: PhantomData<R>,
}

impl<const REPORT_SIZE: usize, Mcu, Messages, Report>
    HidTransport<REPORT_SIZE, Mcu, Messages, Report>
where
    Mcu: mcu::Mcu + CreateDriver,
    Messages: external::Messages,
    Report: AsInputReport + SerializedDescriptor + 'static,
{
    pub fn new(config: TransportConfig, mcu: &'static Mcu) -> Self {
        let device_handler = DeviceHandler::new();

        Self {
            channel: Channel::new(),
            activation_request_signal: Arc::clone(device_handler.activation_request_signal()),
            device_handler: Mutex::new(device_handler),
            config,
            mcu,
            phantom: PhantomData,
        }
    }

    pub async fn run<F: FnMut(Messages) -> Option<Report>>(&self, mut handle_message: F) {
        let driver = self.mcu.create_driver();

        let mut config = embassy_usb::Config::from(self.config.clone());
        config.supports_remote_wakeup = true;

        let mut config_descriptor = [0; 256];
        let mut bos_descriptor = [0; 256];
        let mut msos_descriptor = [0; 256];
        let mut control_buf = [0; 64];

        let mut state = embassy_usb::class::hid::State::new();

        let mut device_handler = self.device_handler.lock().await;
        let suspended = Arc::clone(device_handler.suspended());

        let mut builder = embassy_usb::Builder::new(
            driver,
            config,
            &mut config_descriptor,
            &mut bos_descriptor,
            &mut msos_descriptor,
            &mut control_buf,
        );

        builder.handler(&mut *device_handler);

        let hid_config = embassy_usb::class::hid::Config {
            report_descriptor: Report::desc(),
            request_handler: None,
            poll_ms: 60,
            max_packet_size: 64,
        };
        let mut hid_writer = HidWriter::<_, REPORT_SIZE>::new(&mut builder, &mut state, hid_config);

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
                        if let Err(e) = usb.remote_wakeup().await {
                            error!("Failed to initialize remote wakeup: {}", e);
                        }
                    }
                }
            }
        };

        let write_keyboard_report = async {
            loop {
                let message = self.channel.receive().await;
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

    pub fn send(&self, message: Messages) {
        self.channel.send(message);
    }

    pub fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(self.activation_request_signal.wait())
    }
}
