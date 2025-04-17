use super::Nrf52840;
use crate::external::usb;
use crate::util::{info, unwrap};
use embassy_nrf::bind_interrupts;
use embassy_nrf::interrupt::{InterruptExt, Priority};
use embassy_nrf::peripherals::USBD;
use embassy_nrf::usb::vbus_detect::{SoftwareVbusDetect, VbusDetect};

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
