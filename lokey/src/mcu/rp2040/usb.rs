use super::Rp2040;
use crate::external::{self, Messages1, usb};
use crate::util::channel::Channel;
use crate::util::unwrap;
use crate::{Address, internal};
use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use once_cell::sync::OnceCell;

impl usb::CreateDriver for Rp2040 {
    fn create_driver<'a>(&'static self) -> impl embassy_usb::driver::Driver<'a> {
        bind_interrupts!(struct Irqs {
            USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<embassy_rp::peripherals::USB>;
        });

        let usbd = unsafe { embassy_rp::peripherals::USB::steal() };

        embassy_rp::usb::Driver::new(usbd, Irqs)
    }
}

static CHANNEL: Channel<CriticalSectionRawMutex, external::KeyMessage> = Channel::new();
static ACTIVATION_REQUEST: OnceCell<usb::ActivationRequest> = OnceCell::new();

#[non_exhaustive]
pub struct ExternalTransport {}

impl external::Transport for ExternalTransport {
    type Messages = Messages1<external::KeyMessage>;

    fn send(&self, message: Messages1<external::KeyMessage>) {
        let Messages1::Message1(message) = message;
        CHANNEL.send(message);
    }

    fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(async {
            if let Some(activation_request) = ACTIVATION_REQUEST.get() {
                activation_request.wait().await;
            }
        })
    }
}

impl external::TransportConfig<Rp2040, Messages1<external::KeyMessage>> for usb::TransportConfig {
    type Transport = ExternalTransport;

    async fn init(
        self,
        mcu: &'static Rp2040,
        _address: Address,
        spawner: Spawner,
        _internal_channel: internal::DynChannel,
    ) -> Self::Transport {
        if ACTIVATION_REQUEST.get().is_some() {
            // Channel was already intialized
            return ExternalTransport {};
        }

        let (handler, activation_request) = usb::Handler::new(self, mcu);
        unwrap!(spawner.make_send().spawn(task(handler)));
        let _ = ACTIVATION_REQUEST.set(activation_request);
        ExternalTransport {}
    }
}

#[embassy_executor::task]
async fn task(handler: usb::Handler<Rp2040>) {
    handler.run(CHANNEL.receiver()).await
}
