use super::{CreateDriver, DeviceHandler, TransportConfig};
use crate::util::channel::Channel;
use crate::util::{error, info};
use crate::{external, mcu};
use alloc::boxed::Box;
use arrayvec::ArrayVec;
use core::pin::Pin;
use core::sync::atomic::Ordering;
use embassy_futures::join::{join, join3};
use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_usb::class::hid::{HidReaderWriter, HidWriter, ReadError};
use portable_atomic_util::Arc;

pub struct HidWriteTransport<const WRITE_REPORT_SIZE: usize, Mcu: 'static, TxMessages> {
    tx_channel: Channel<CriticalSectionRawMutex, TxMessages>,
    device_handler: Mutex<CriticalSectionRawMutex, DeviceHandler>,
    activation_request_signal: Arc<Signal<CriticalSectionRawMutex, ()>>,
    config: TransportConfig,
    mcu: &'static Mcu,
}

impl<const WRITE_REPORT_SIZE: usize, Mcu: mcu::Mcu + CreateDriver + 'static, TxMessages>
    HidWriteTransport<WRITE_REPORT_SIZE, Mcu, TxMessages>
{
    pub fn new(config: TransportConfig, mcu: &'static Mcu) -> Self {
        let device_handler = DeviceHandler::new();

        Self {
            tx_channel: Channel::new(),
            activation_request_signal: Arc::clone(device_handler.activation_request_signal()),
            device_handler: Mutex::new(device_handler),
            config,
            mcu,
        }
    }

    pub async fn run<F>(&self, write_report_descriptor: &[u8], mut handle_message: F)
    where
        F: FnMut(TxMessages) -> Option<ArrayVec<u8, WRITE_REPORT_SIZE>>,
    {
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
            report_descriptor: write_report_descriptor,
            request_handler: None,
            poll_ms: 60,
            max_packet_size: 64,
        };
        let mut hid_writer =
            HidWriter::<_, WRITE_REPORT_SIZE>::new(&mut builder, &mut state, hid_config);

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

        let write_report = async {
            loop {
                let message = self.tx_channel.receive().await;
                if suspended.load(Ordering::Acquire) {
                    info!("Triggering remote wakeup");
                    remote_wakeup.signal(());
                } else if let Some(report) = handle_message(message) {
                    if let Err(e) = hid_writer.write(&report).await {
                        error!("Failed to write report: {}", e);
                    }
                }
            }
        };

        join(wakeup, write_report).await.0
    }

    pub fn send(&self, message: TxMessages) {
        self.tx_channel.send(message);
    }

    pub fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(self.activation_request_signal.wait())
    }
}

pub struct HidReadWriteTransport<
    const WRITE_REPORT_SIZE: usize,
    const READ_REPORT_SIZE: usize,
    Mcu: 'static,
    TxMessages,
    RxMessages,
> {
    tx_channel: Channel<CriticalSectionRawMutex, TxMessages>,
    rx_channel: Channel<CriticalSectionRawMutex, RxMessages>,
    device_handler: Mutex<CriticalSectionRawMutex, DeviceHandler>,
    activation_request_signal: Arc<Signal<CriticalSectionRawMutex, ()>>,
    config: TransportConfig,
    mcu: &'static Mcu,
}

impl<const WRITE_REPORT_SIZE: usize, const READ_REPORT_SIZE: usize, Mcu, TxMessages, RxMessages>
    HidReadWriteTransport<WRITE_REPORT_SIZE, READ_REPORT_SIZE, Mcu, TxMessages, RxMessages>
where
    Mcu: mcu::Mcu + CreateDriver,
    TxMessages: external::TxMessages,
    RxMessages: external::RxMessages,
{
    pub fn new(config: TransportConfig, mcu: &'static Mcu) -> Self {
        let device_handler = DeviceHandler::new();

        Self {
            tx_channel: Channel::new(),
            rx_channel: Channel::new(),
            activation_request_signal: Arc::clone(device_handler.activation_request_signal()),
            device_handler: Mutex::new(device_handler),
            config,
            mcu,
        }
    }

    pub async fn run<F1, F2>(
        &self,
        write_report_descriptor: &[u8],
        mut handle_message: F1,
        mut handle_report: F2,
    ) where
        F1: FnMut(TxMessages) -> Option<ArrayVec<u8, WRITE_REPORT_SIZE>>,
        F2: FnMut(ArrayVec<u8, READ_REPORT_SIZE>) -> Option<RxMessages>,
    {
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
            report_descriptor: write_report_descriptor,
            request_handler: None,
            poll_ms: 60,
            max_packet_size: 64,
        };
        let hid_reader_writer = HidReaderWriter::<_, READ_REPORT_SIZE, WRITE_REPORT_SIZE>::new(
            &mut builder,
            &mut state,
            hid_config,
        );
        let (mut hid_reader, mut hid_writer) = hid_reader_writer.split();

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

        let write_report = async {
            loop {
                let message = self.tx_channel.receive().await;
                if suspended.load(Ordering::Acquire) {
                    info!("Triggering remote wakeup");
                    remote_wakeup.signal(());
                } else if let Some(report) = handle_message(message) {
                    if let Err(e) = hid_writer.write(&report).await {
                        error!("Failed to write report: {}", e);
                    }
                }
            }
        };

        let read_report = async {
            loop {
                let mut buf = ArrayVec::from([0; READ_REPORT_SIZE]);
                match hid_reader.read(&mut buf).await {
                    Ok(len) => {
                        buf.truncate(len);
                        if let Some(message) = handle_report(buf) {
                            self.rx_channel.send(message);
                        }
                    }
                    Err(ReadError::BufferOverflow) => error!(
                        "Host sent output report larger than the configured maximum output report length ({})",
                        READ_REPORT_SIZE
                    ),
                    Err(ReadError::Disabled) => error!("Endpoint is disabled"),
                    Err(ReadError::Sync(_)) => unreachable!(),
                }
            }
        };

        join3(wakeup, write_report, read_report).await.0
    }

    pub fn send(&self, message: TxMessages) {
        self.tx_channel.send(message);
    }

    pub async fn receive(&self) -> RxMessages {
        self.rx_channel.receive().await
    }

    pub fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(self.activation_request_signal.wait())
    }
}
