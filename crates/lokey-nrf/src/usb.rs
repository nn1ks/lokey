use crate::Nrf;
use embassy_nrf::interrupt::{InterruptExt, Priority};
use embassy_nrf::peripherals::USBD;
use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;
use lokey_usb::CreateDriver;

impl CreateDriver for Nrf {
    type Driver<'d> = embassy_nrf::usb::Driver<'d, HardwareVbusDetect>;

    fn create_driver<'d>(&'static self) -> Self::Driver<'d> {
        embassy_nrf::interrupt::USBD.set_priority(Priority::P2);

        let usbd = unsafe { USBD::steal() };

        let vbus_detect = HardwareVbusDetect::new(super::Irqs);
        embassy_nrf::usb::Driver::new(usbd, super::Irqs, vbus_detect)
    }
}
