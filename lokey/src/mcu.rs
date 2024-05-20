#[cfg(feature = "nrf52840")]
pub mod nrf52840;
pub mod storage;

#[cfg(feature = "nrf52840")]
pub use nrf52840::Nrf52840;
pub use storage::Storage;

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

// This is only used for doc tests
#[doc(hidden)]
pub use dummy::DummyMcu;

mod dummy {
    use super::*;

    pub struct DummyMcu;

    impl Mcu for DummyMcu {}

    impl McuInit for DummyMcu {
        type Config = ();

        fn create(_config: Self::Config, _spawner: Spawner) -> Self
        where
            Self: Sized,
        {
            Self
        }

        fn run(&'static self, _spawner: Spawner) {}
    }

    impl HeapSize for DummyMcu {
        const DEFAULT_HEAP_SIZE: usize = 0;
    }
}
