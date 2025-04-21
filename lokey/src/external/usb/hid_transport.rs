use super::{CreateDriver, DeviceHandler, Messages, Transport, TransportConfig};
use crate::mcu::Mcu;
use crate::util::channel::Channel;
use crate::util::{error, info, unwrap};
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
use portable_atomic_util::Arc;
use usbd_hid::descriptor::{AsInputReport, SerializedDescriptor};

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
                            if let Err(e) = usb.remote_wakeup().await {
                                error!("Failed to initialize remote wakeup: {}", e);
                            }
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
