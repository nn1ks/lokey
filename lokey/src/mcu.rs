use crate::{Address, Context, Device, StateContainer, Transports};
use core::any::Any;

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
