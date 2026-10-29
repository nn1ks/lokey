use crate::{Address, AnyState, Context, Device, Transports, storage};
use core::any::Any;

/// Creates an MCU together with its storage backend.
///
/// A single MCU can implement this trait multiple times, once per storage configuration, which
/// allows the same hardware to be used with different storage backends. The [`Device`]
/// implementation selects one of them via [`Device::StorageConfig`].
///
/// This trait is called by the [`device`](crate::device) macro. The created MCU and storage
/// instance are provided via the [`Context`].
pub trait McuCreator<StorageConfig>: Mcu {
    /// The storage backend created alongside this MCU.
    type Storage: storage::Storage;

    /// Initializes the MCU and creates its storage backend.
    ///
    /// This function must be called only once per concrete MCU type.
    fn create(
        spawner: Spawner,
        mcu_config: Self::Config,
        address: Address,
        storage_config: StorageConfig,
    ) -> impl Future<Output = (Self, Self::Storage)>
    where
        Self: Sized;
}

/// Microcontroller abstraction.
///
/// This trait encapsulates Microcontroller-specific setup and background execution.
pub trait Mcu: Any {
    /// The configuration for this MCU.
    type Config: Default;

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
        Self: Sized + McuCreator<D::StorageConfig>;
}

pub use dummy::DummyMcu;
use embassy_executor::Spawner;

#[allow(missing_docs)]
mod dummy {
    use super::*;
    use crate::storage::{EmptyStorage, EmptyStorageConfig};

    pub struct DummyMcu;

    impl Mcu for DummyMcu {
        type Config = ();

        async fn run<D, T, S>(&'static self, _context: Context<D, T, S>)
        where
            D: Device<Mcu = Self>,
            T: Transports<Self>,
            S: AnyState,
            Self: McuCreator<D::StorageConfig>,
        {
        }
    }

    impl McuCreator<EmptyStorageConfig> for DummyMcu {
        type Storage = EmptyStorage;

        async fn create(
            _: Spawner,
            _: Self::Config,
            _: Address,
            _: EmptyStorageConfig,
        ) -> (Self, Self::Storage)
        where
            Self: Sized,
        {
            (Self, EmptyStorage)
        }
    }
}
