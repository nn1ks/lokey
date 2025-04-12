#[cfg(feature = "nrf52840")]
pub mod nrf52840;
pub mod pwm;
#[cfg(feature = "rp2040")]
pub mod rp2040;
pub mod storage;

use crate::DynContext;
use core::any::Any;
use embassy_executor::Spawner;
#[cfg(feature = "nrf52840")]
pub use nrf52840::Nrf52840;
#[cfg(feature = "rp2040")]
pub use rp2040::Rp2040;
pub use storage::Storage;

pub trait Mcu: Any {}

pub trait McuInit: Mcu {
    /// The configuration for this MCU.
    type Config;

    /// Creates the MCU.
    ///
    /// This function must be called only once for a MCU type.
    fn create(config: Self::Config, spawner: Spawner) -> Self
    where
        Self: Sized;

    /// Runs MCU specific tasks.
    ///
    /// This function must be called only once for a MCU type.
    fn run(&'static self, context: DynContext);
}

pub trait McuStorage<F> {
    fn storage(&self) -> &'static Storage<F>;
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

        fn create(_config: Self::Config, _spawner: Spawner) -> Self {
            Self
        }

        fn run(&'static self, _context: DynContext) {}
    }

    impl HeapSize for DummyMcu {
        const DEFAULT_HEAP_SIZE: usize = 0;
    }
}
