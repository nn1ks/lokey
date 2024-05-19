use super::Nrf52840;
use crate::external::{self, usb};
use crate::internal;
use alloc::boxed::Box;
use core::future::Future;
use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::interrupt::{InterruptExt, Priority};
use embassy_nrf::usb::vbus_detect::{SoftwareVbusDetect, VbusDetect};
use embassy_nrf::{bind_interrupts, peripherals::USBD};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

struct SoftwareVbusDetectWrapper(SoftwareVbusDetect);

impl VbusDetect for SoftwareVbusDetectWrapper {
    fn is_usb_detected(&self) -> bool {
        (&self.0).is_usb_detected()
    }

    async fn wait_power_ready(&mut self) -> Result<(), ()> {
        (&self.0).wait_power_ready().await
    }
}

impl usb::Driver for Nrf52840 {
    fn driver<'a>(&'static self) -> impl embassy_usb::driver::Driver<'a> {
        bind_interrupts!(struct Irqs {
            USBD => embassy_nrf::usb::InterruptHandler<embassy_nrf::peripherals::USBD>;
            POWER_CLOCK => embassy_nrf::usb::vbus_detect::InterruptHandler;
        });

        embassy_nrf::interrupt::USBD.set_priority(Priority::P2);
        embassy_nrf::interrupt::POWER_CLOCK.set_priority(Priority::P2);

        info!("Enabling ext hfosc...");
        nrf_softdevice::RawError::convert(unsafe { nrf_softdevice::raw::sd_clock_hfclk_request() })
            .unwrap();
        let mut is_running = 0;
        while is_running != 1 {
            nrf_softdevice::RawError::convert(unsafe {
                nrf_softdevice::raw::sd_clock_hfclk_is_running(&mut is_running)
            })
            .unwrap();
        }

        let usbd = unsafe { USBD::steal() };

        let vbus_detect = SoftwareVbusDetectWrapper(SoftwareVbusDetect::new(true, true));
        embassy_nrf::usb::Driver::new(usbd, Irqs, vbus_detect)
    }
}

static CHANNEL: embassy_sync::channel::Channel<CriticalSectionRawMutex, external::Message, 8> =
    embassy_sync::channel::Channel::new();

pub struct ExternalChannel {
    _private: (),
}

impl external::ChannelImpl for ExternalChannel {
    fn send(&self, message: external::Message) -> Box<dyn Future<Output = ()> + '_> {
        Box::new(async {
            CHANNEL.send(message).await;
        })
    }

    fn request_active(&self) -> Box<dyn Future<Output = ()> + '_> {
        // TODO
        Box::new(core::future::pending())
    }
}

impl external::ChannelConfig<Nrf52840> for usb::ChannelConfig {
    type Channel = ExternalChannel;

    async fn init(
        self,
        mcu: &'static Nrf52840,
        spawner: Spawner,
        _internal_channel: internal::DynChannel,
    ) -> Self::Channel {
        unwrap!(spawner.spawn(task(self, mcu)));
        ExternalChannel { _private: () }
    }
}

#[embassy_executor::task]
async fn task(config: usb::ChannelConfig, mcu: &'static Nrf52840) {
    usb::common(config, mcu, CHANNEL.receiver()).await
}
