#[cfg(feature = "usb")]
pub mod usb;

use super::{HeapSize, Mcu, McuInit, McuStorage, Storage};
use crate::{DynContext, external, internal};
use alloc::boxed::Box;
use core::ops::Range;
use embassy_executor::Spawner;
use embassy_rp::flash;
use embassy_rp::peripherals::{DMA_CH0, FLASH};

pub struct Config {
    pub storage_flash_range: Range<u32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage_flash_range: 0..0x1_0000,
        }
    }
}

pub type Flash = flash::Flash<'static, FLASH, flash::Async, 0x200000>;

pub struct Rp2040 {
    storage: &'static Storage<Flash>,
}

impl Mcu for Rp2040 {}

impl McuInit for Rp2040 {
    type Config = Config;

    fn create<E, I>(
        config: Self::Config,
        _external_transport_config: &E,
        _internal_transport_config: &I,
        _spawner: Spawner,
    ) -> Self
    where
        Self: Sized,
        E: external::TransportConfig<Self> + 'static,
        I: internal::TransportConfig<Self> + 'static,
    {
        let rp_config = embassy_rp::config::Config::default();
        embassy_rp::init(rp_config);
        let flash = Flash::new(unsafe { FLASH::steal() }, unsafe { DMA_CH0::steal() });
        let storage = Storage::new(flash, config.storage_flash_range);
        Self {
            storage: Box::leak(Box::new(storage)),
        }
    }

    fn run(&'static self, _context: DynContext) {}
}

impl McuStorage<Flash> for Rp2040 {
    fn storage(&self) -> &'static Storage<Flash> {
        self.storage
    }
}

impl HeapSize for Rp2040 {
    // The RP2040 has 264kB of RAM
    const DEFAULT_HEAP_SIZE: usize = 64 * 1024; // 64kB
}
