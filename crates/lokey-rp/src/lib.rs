//! Raspberry Pi RP2040 and RP235x microcontroller support for the lokey framework.
//!
//! # Feature flags
//!
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
//!

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

use embassy_executor::Spawner;
use embassy_rp::clocks::RoscRng;
use embassy_rp::peripherals::{DMA_CH0, FLASH};
use embassy_rp::{bind_interrupts, flash};
use lokey::storage::flash::MultiwriteNorFlash;
use lokey::storage::{
    self, DefaultStorage, DefaultStorageConfig, EmptyStorage, EmptyStorageConfig,
};
use lokey::{Address, AnyState, Context, Device, Mcu, McuCreator, Transports};

bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>;
    #[cfg(feature = "usb")]
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<embassy_rp::peripherals::USB>;
});

#[non_exhaustive]
pub struct Rp {}

impl Mcu for Rp {
    type Config = embassy_rp::config::Config;

    async fn run<D, T, S>(_context: Context<D, T, S>)
    where
        D: Device<Mcu = Self>,
        T: Transports<Self>,
        S: AnyState,
        Self: McuCreator<D::StorageConfig>,
    {
    }
}

type Flash = flash::Flash<'static, FLASH, flash::Async, 0x200000>;

impl McuCreator<DefaultStorageConfig> for Rp {
    type Storage = DefaultStorage<MultiwriteNorFlash<Flash>>;

    async fn create(
        _: Spawner,
        mcu_config: Self::Config,
        _: Address,
        storage_config: DefaultStorageConfig,
    ) -> (Self, Self::Storage) {
        let p = embassy_rp::init(mcu_config);

        let storage = {
            let flash = Flash::new(p.FLASH, p.DMA_CH0, Irqs);
            let flash = MultiwriteNorFlash::new(flash, storage_config.flash_page_range);
            let random_seed = RoscRng.next_u32();
            DefaultStorage::new(flash, random_seed)
        };
        let _ = storage::mount_or_format(&storage).await;

        (Self {}, storage)
    }
}

impl McuCreator<EmptyStorageConfig> for Rp {
    type Storage = EmptyStorage;

    async fn create(
        _: Spawner,
        mcu_config: Self::Config,
        _: Address,
        _: EmptyStorageConfig,
    ) -> (Self, Self::Storage) {
        embassy_rp::init(mcu_config);

        (Self {}, EmptyStorage)
    }
}

#[cfg(feature = "usb")]
mod usb {
    use super::{Irqs, Rp};
    use lokey_usb::CreateDriver;

    impl CreateDriver for Rp {
        type Driver<'d> = embassy_rp::usb::Driver<'d, embassy_rp::peripherals::USB>;

        fn create_driver<'d>(&'static self) -> Self::Driver<'d> {
            let usbd = unsafe { embassy_rp::peripherals::USB::steal() };

            embassy_rp::usb::Driver::new(usbd, Irqs)
        }
    }
}
