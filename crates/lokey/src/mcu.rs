use crate::{Address, AnyState, Context, Device, Transports};
use core::any::Any;

/// Microcontroller abstraction.
///
/// This trait encapsulates Microcontroller-specific setup and background execution.
pub trait Mcu: Any {
    /// The configuration for this MCU.
    type Config: Default;

    /// Creates and initializes the MCU instance.
    ///
    /// This function must be called only once per concrete MCU type.
    fn create(config: Self::Config, address: Address) -> impl Future<Output = Self>
    where
        Self: Sized;

    /// Runs MCU-specific tasks.
    ///
    /// This function is expected to drive MCU background work and usually runs for the lifetime of
    /// the device.
    ///
    /// This function must be called only once per concrete MCU type.
    fn run<D, T, S>(&'static self, context: Context<D, T, S>) -> impl Future<Output = ()>
    where
        D: Device<Mcu = Self>,
        T: Transports<Self>,
        S: AnyState,
        Self: Sized;
}

pub use dummy::DummyMcu;

#[allow(missing_docs)]
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
            S: AnyState,
        {
        }
    }
}
