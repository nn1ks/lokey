#[cfg(feature = "usb")]
pub mod usb;

use super::{HeapSize, Mcu, McuInit};
use crate::DynContext;
use embassy_executor::Spawner;

// TODO: Storage

pub struct Config;

pub struct Rp2040 {}

impl Mcu for Rp2040 {}

impl McuInit for Rp2040 {
    type Config = Config;

    fn create(_config: Self::Config, _spawner: Spawner) -> Self
    where
        Self: Sized,
    {
        let config = embassy_rp::config::Config::default();
        embassy_rp::init(config);
        Self {}
    }

    fn run(&'static self, _context: DynContext) {}
}

impl HeapSize for Rp2040 {
    // The RP2040 has 264kB of RAM
    const DEFAULT_HEAP_SIZE: usize = 64 * 1024; // 64kB
}
