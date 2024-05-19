#[cfg(feature = "nrf52840")]
pub mod nrf52840;

#[cfg(feature = "nrf52840")]
pub use nrf52840::Nrf52840;

use core::any::Any;
use embassy_executor::Spawner;

pub trait Mcu: Any {}

pub trait McuInit {
    type Config;
    fn create(config: Self::Config, spawner: Spawner) -> Self
    where
        Self: Sized;
    fn run(&'static self, spawner: Spawner);
}

pub trait HeapSize {
    const DEFAULT_HEAP_SIZE: usize;
}
