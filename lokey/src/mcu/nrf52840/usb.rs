use super::Nrf52840;
use crate::external::usb;
use embassy_nrf::interrupt::{InterruptExt, Priority};
use embassy_nrf::peripherals::USBD;
use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;

impl usb::CreateDriver for Nrf52840 {
    fn create_driver<'a>(&'static self) -> impl embassy_usb::driver::Driver<'a> {
        embassy_nrf::interrupt::USBD.set_priority(Priority::P2);
        embassy_nrf::interrupt::CLOCK_POWER.set_priority(Priority::P2);

        let usbd = unsafe { USBD::steal() };

        let vbus_detect = HardwareVbusDetect::new(super::Irqs);
        embassy_nrf::usb::Driver::new(usbd, super::Irqs, vbus_detect)
    }
}
