//! Raspberry Pi RP2040 and RP235x microcontroller support for the lokey framework.
//!
//! # Feature flags
//!
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
//!

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

use core::ops::Range;
use embassy_rp::flash;
use embassy_rp::peripherals::{DMA_CH0, FLASH};
use lokey::storage::flash::MultiwriteNorFlash;
use lokey::storage::{DefaultStorage, StorageDriver};
use lokey::{Address, AnyState, Context, Device, Mcu, Transports};

pub struct StorageConfig {
    pub flash_page_range: Range<usize>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            flash_page_range: 0..0x100,
        }
    }
}

#[non_exhaustive]
pub struct Rp {}

impl Mcu for Rp {
    type Config = embassy_rp::config::Config;

    async fn create(config: Self::Config, _address: Address) -> Self {
        embassy_rp::init(config);
        Self {}
    }

    async fn run<D, T, S>(&'static self, _context: Context<D, T, S>)
    where
        D: Device<Mcu = Self>,
        T: Transports<Self>,
        S: AnyState,
    {
    }
}

type Flash = flash::Flash<'static, FLASH, flash::Async, 0x200000>;

pub struct DefaultStorageDriver;

impl StorageDriver for DefaultStorageDriver {
    type Storage = DefaultStorage<MultiwriteNorFlash<Flash>>;
    type Mcu = Rp;
    type Config = StorageConfig;

    fn create_storage(_: &'static Self::Mcu, config: Self::Config) -> Self::Storage {
        let flash = Flash::new(unsafe { FLASH::steal() }, unsafe { DMA_CH0::steal() });
        let flash = MultiwriteNorFlash::new(flash, config.flash_page_range);
        let random_seed = 0; // TODO
        DefaultStorage::new(flash, random_seed)
    }
}

#[cfg(feature = "usb")]
mod usb {
    use super::Rp;
    use embassy_rp::bind_interrupts;
    use lokey_usb::CreateDriver;

    impl CreateDriver for Rp {
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
