use crate::StorageConfig;
use embassy_rp::flash;
use embassy_rp::peripherals::{DMA_CH0, FLASH};
use lokey::storage::{DefaultStorage, StorageDriver};
use lokey::{Address, Context, Device, Mcu, StateContainer, Transports};

#[derive(Default)]
pub struct Config {}

#[non_exhaustive]
pub struct Rp2040 {}

impl Mcu for Rp2040 {
    type Config = Config;

    async fn create(_: Self::Config, _address: Address) -> Self {
        let rp_config = embassy_rp::config::Config::default();
        embassy_rp::init(rp_config);
        Self {}
    }

    async fn run<D, T, S>(&'static self, _context: Context<D, T, S>)
    where
        D: Device<Mcu = Self>,
        T: Transports<Self>,
        S: StateContainer,
    {
    }
}

type Flash = flash::Flash<'static, FLASH, flash::Async, 0x200000>;

type WordSize = typenum::U4;
type EraseSize = typenum::U4096;

pub struct DefaultStorageDriver;

impl StorageDriver for DefaultStorageDriver {
    type Storage = DefaultStorage<Flash, WordSize, EraseSize>;
    type Mcu = Rp2040;
    type Config = StorageConfig;

    fn create_storage(_: &'static Self::Mcu, config: Self::Config) -> Self::Storage {
        let flash = Flash::new(unsafe { FLASH::steal() }, unsafe { DMA_CH0::steal() });
        DefaultStorage::new(flash, config.flash_range)
    }
}

#[cfg(feature = "usb")]
mod usb {
    use super::Rp2040;
    use embassy_rp::bind_interrupts;
    use lokey_usb::CreateDriver;

    impl CreateDriver for Rp2040 {
        type Driver<'d> = embassy_rp::usb::Driver<'d, embassy_rp::peripherals::USB>;

        fn create_driver<'d>(&'static self) -> Self::Driver<'d> {
            bind_interrupts!(struct Irqs {
                USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<embassy_rp::peripherals::USB>;
            });

            let usbd = unsafe { embassy_rp::peripherals::USB::steal() };

            embassy_rp::usb::Driver::new(usbd, Irqs)
        }
    }
}
