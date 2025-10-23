use super::{CreateDriver, TransportConfig};
use crate::external::MessageServiceRegistry;
use crate::external::usb::{
    self, DeviceHandlerContext, InitMessageService, RxMessageService, TxMessageService,
};
use crate::util::{error, info, unwrap};
use crate::{Address, external, internal, mcu};
use core::sync::atomic::Ordering;
use embassy_futures::join::join3;
use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;

pub struct Transport<Mcu: 'static, TxMessage, RxMessage> {
    tx_channel: Channel<CriticalSectionRawMutex, TxMessage, 1>,
    rx_channel: Channel<CriticalSectionRawMutex, RxMessage, 1>,
    device_handler_context: DeviceHandlerContext,
    config: TransportConfig,
    mcu: &'static Mcu,
}

impl<Mcu, TxMessage, RxMessage> external::Transport for Transport<Mcu, TxMessage, RxMessage>
where
    Mcu: mcu::Mcu + CreateDriver,
    TxMessage: usb::TxMessage,
    RxMessage: usb::RxMessage,
{
    type Config = TransportConfig;
    type Mcu = Mcu;
    type TxMessage = TxMessage;
    type RxMessage = RxMessage;

    async fn create<T: internal::Transport<Mcu = Self::Mcu>>(
        config: Self::Config,
        mcu: &'static Self::Mcu,
        _: Address,
        _: &'static internal::Channel<T>,
    ) -> Self {
        let device_handler_context = DeviceHandlerContext::new();

        Self {
            tx_channel: Channel::new(),
            rx_channel: Channel::new(),
            device_handler_context,
            config,
            mcu,
        }
    }

    async fn run(&self) {
        let driver = self.mcu.create_driver();

        let mut config = embassy_usb::Config::from(self.config.clone());
        config.supports_remote_wakeup = true;

        let mut config_descriptor = [0; 256];
        let mut bos_descriptor = [0; 256];
        let mut msos_descriptor = [0; 256];
        let mut control_buf = [0; 64];

        let mut device_handler = self.device_handler_context.create_device_handler();

        let mut builder = embassy_usb::Builder::new(
            driver,
            config,
            &mut config_descriptor,
            &mut bos_descriptor,
            &mut msos_descriptor,
            &mut control_buf,
        );

        builder.handler(&mut device_handler);

        let mut message_service_registry = MessageServiceRegistry::new();

        TxMessage::MessageService::init(&mut message_service_registry, &mut builder);
        RxMessage::MessageService::init(&mut message_service_registry, &mut builder);
        let tx_message_service = unwrap!(
            message_service_registry.get::<TxMessage::MessageService<'_, Mcu::Driver<'_>>>()
        );
        let rx_message_service = unwrap!(
            message_service_registry.get::<RxMessage::MessageService<'_, Mcu::Driver<'_>>>()
        );

        let mut usb = builder.build();

        let remote_wakeup: Signal<CriticalSectionRawMutex, ()> = Signal::new();

        let wakeup = async {
            loop {
                usb.run_until_suspend().await;
                match select(usb.wait_resume(), remote_wakeup.wait()).await {
                    Either::First(()) => {
                        self.device_handler_context
                            .suspended
                            .store(false, Ordering::Release);
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
                let suspended = {
                    self.device_handler_context
                        .suspended
                        .load(Ordering::Acquire)
                };
                if suspended {
                    info!("Triggering remote wakeup");
                    remote_wakeup.signal(());
                } else {
                    tx_message_service.send(message).await;
                }
            }
        };

        let read_report = async {
            loop {
                let message = rx_message_service.receive().await;
                self.rx_channel.send(message).await;
            }
        };

        join3(wakeup, write_report, read_report).await.0
    }

    async fn send(&self, message: Self::TxMessage) {
        self.tx_channel.send(message).await;
    }

    async fn receive(&self) -> Self::RxMessage {
        self.rx_channel.receive().await
    }

    async fn wait_for_activation_request(&self) {
        self.device_handler_context
            .activation_request_signal
            .wait()
            .await
    }
}
