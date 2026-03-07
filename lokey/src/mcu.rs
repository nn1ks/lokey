pub mod storage;

use crate::{Address, Context, Device, StateContainer, Transports};
use core::any::Any;
use embedded_storage_async::nor_flash::MultiwriteNorFlash;
use generic_array::ArrayLength;
pub use storage::Storage;

pub trait Mcu: Any {
    /// The configuration for this MCU.
    type Config;

    /// Creates the MCU.
    ///
    /// This function must be called only once for a MCU type.
    fn create(config: Self::Config, address: Address) -> impl Future<Output = Self>
    where
        Self: Sized;

    /// Runs MCU specific tasks.
    ///
    /// This function must be called only once for a MCU type.
    fn run<D, T, S>(&'static self, context: Context<D, T, S>) -> impl Future<Output = ()>
    where
        D: Device<Mcu = Self>,
        T: Transports<Self>,
        S: StateContainer,
        Self: Sized;
}

pub trait McuStorage {
    type Flash: MultiwriteNorFlash;
    type WordSize: ArrayLength;
    type EraseSize: ArrayLength;
    fn storage(&self) -> &Storage<Self::Flash, Self::WordSize, Self::EraseSize>;
}

// This is only used for doc tests
#[doc(hidden)]
pub use dummy::DummyMcu;

mod dummy {
    use super::*;

    pub struct DummyMcu;

    impl Mcu for DummyMcu {
        type Config = ();

        async fn create(_config: Self::Config, _address: Address) -> Self {
            Self
        }

        async fn run<D, T, S>(&'static self, _context: Context<D, T, S>)
        where
            D: Device<Mcu = Self>,
            T: Transports<Self>,
            S: StateContainer,
        {
        }
    }
}
