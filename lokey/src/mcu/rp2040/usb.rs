use super::Rp2040;
use crate::external::usb;
use embassy_rp::bind_interrupts;

impl usb::CreateDriver for Rp2040 {
    fn create_driver<'a>(&'static self) -> impl embassy_usb::driver::Driver<'a> {
        bind_interrupts!(struct Irqs {
            USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<embassy_rp::peripherals::USB>;
        });

        let usbd = unsafe { embassy_rp::peripherals::USB::steal() };

        embassy_rp::usb::Driver::new(usbd, Irqs)
    }
}
