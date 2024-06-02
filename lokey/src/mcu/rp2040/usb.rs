use super::Rp2040;
use crate::external::{self, usb};
use crate::{internal, util};
use alloc::boxed::Box;
use core::{future::Future, pin::Pin};
use defmt::unwrap;
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

static CHANNEL: util::channel::Channel<CriticalSectionRawMutex, external::Message> =
    util::channel::Channel::new();
static ACTIVATION_REQUEST: OnceCell<usb::ActivationRequest> = OnceCell::new();

pub struct ExternalChannel {}

impl external::ChannelImpl for ExternalChannel {
    fn send(&self, message: external::Message) {
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

impl external::ChannelConfig<Rp2040> for usb::ChannelConfig {
    type Channel = ExternalChannel;

    async fn init(
        self,
        mcu: &'static Rp2040,
        spawner: Spawner,
        _internal_channel: internal::DynChannel,
    ) -> Self::Channel {
        if ACTIVATION_REQUEST.get().is_some() {
            // Channel was already intialized
            return ExternalChannel {};
        }

        let (handler, activation_request) = usb::Handler::new(self, mcu);
        unwrap!(spawner.spawn(task(handler)));
        let _ = ACTIVATION_REQUEST.set(activation_request);
        ExternalChannel {}
    }
}

#[embassy_executor::task]
async fn task(handler: usb::Handler<Rp2040>) {
    handler.run(CHANNEL.receiver()).await
}
