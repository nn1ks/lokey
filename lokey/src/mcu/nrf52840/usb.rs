use super::Nrf52840;
use crate::external::{self, usb};
use crate::{internal, util::channel::Channel};
use alloc::boxed::Box;
use core::{future::Future, pin::Pin};
use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::interrupt::{InterruptExt, Priority};
use embassy_nrf::usb::vbus_detect::{SoftwareVbusDetect, VbusDetect};
use embassy_nrf::{bind_interrupts, peripherals::USBD};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use once_cell::sync::OnceCell;

struct SoftwareVbusDetectWrapper(SoftwareVbusDetect);

impl VbusDetect for SoftwareVbusDetectWrapper {
    fn is_usb_detected(&self) -> bool {
        (&self.0).is_usb_detected()
    }

    async fn wait_power_ready(&mut self) -> Result<(), ()> {
        (&self.0).wait_power_ready().await
    }
}

impl usb::CreateDriver for Nrf52840 {
    fn create_driver<'a>(&'static self) -> impl embassy_usb::driver::Driver<'a> {
        bind_interrupts!(struct Irqs {
            USBD => embassy_nrf::usb::InterruptHandler<embassy_nrf::peripherals::USBD>;
            CLOCK_POWER => embassy_nrf::usb::vbus_detect::InterruptHandler;
        });

        embassy_nrf::interrupt::USBD.set_priority(Priority::P2);
        embassy_nrf::interrupt::CLOCK_POWER.set_priority(Priority::P2);

        info!("Enabling ext hfosc...");
        unwrap!(nrf_softdevice::RawError::convert(unsafe {
            nrf_softdevice::raw::sd_clock_hfclk_request()
        }));
        let mut is_running = 0;
        while is_running != 1 {
            unwrap!(nrf_softdevice::RawError::convert(unsafe {
                nrf_softdevice::raw::sd_clock_hfclk_is_running(&mut is_running)
            }));
        }

        let usbd = unsafe { USBD::steal() };

        let vbus_detect = SoftwareVbusDetectWrapper(SoftwareVbusDetect::new(true, true));
        embassy_nrf::usb::Driver::new(usbd, Irqs, vbus_detect)
    }
}

static CHANNEL: Channel<CriticalSectionRawMutex, external::Message> = Channel::new();
static ACTIVATION_REQUEST: OnceCell<usb::ActivationRequest> = OnceCell::new();

#[non_exhaustive]
pub struct ExternalTransport {}

impl external::Transport for ExternalTransport {
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

impl external::TransportConfig<Nrf52840> for usb::TransportConfig {
    type Transport = ExternalTransport;

    async fn init(
        self,
        mcu: &'static Nrf52840,
        spawner: Spawner,
        _internal_channel: internal::DynChannel,
    ) -> Self::Transport {
        if ACTIVATION_REQUEST.get().is_some() {
            // Channel was already intialized
            return ExternalTransport {};
        }

        let (handler, activation_request) = usb::Handler::new(self, mcu);
        unwrap!(spawner.spawn(task(handler)));
        let _ = ACTIVATION_REQUEST.set(activation_request);
        ExternalTransport {}
    }
}

#[embassy_executor::task]
async fn task(handler: usb::Handler<Nrf52840>) {
    handler.run(CHANNEL.receiver()).await
}
