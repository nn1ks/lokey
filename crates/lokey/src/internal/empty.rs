//! Empty internal transport implementation.

use crate::{Address, AnyState, Context, McuCreator, internal, storage};
use core::marker::PhantomData;

/// Configuration for the empty internal transport.
pub struct TransportConfig;

/// An internal transport implementation that does not actually send or receive any messages.
///
/// This transport is intended for single-part devices that do not require internal communication
/// between devices.
pub struct Transport<Mcu> {
    phantom: PhantomData<Mcu>,
}

impl<Mcu> internal::Transport for Transport<Mcu>
where
    Mcu: 'static,
{
    type Config = TransportConfig;
    type Mcu = Mcu;

    async fn create<Storage>(
        _: Self::Config,
        _: Address,
        _: &'static Self::Mcu,
        _: &'static Storage,
    ) -> Self
    where
        Storage: storage::Storage,
    {
        Self {
            phantom: PhantomData,
        }
    }

    async fn run<Device, Transports, State>(&self, _: Context<Device, Transports, State>)
    where
        Device: crate::Device<Mcu = Self::Mcu>,
        Transports: crate::Transports<Self::Mcu, InternalTransport = Self>,
        State: AnyState,
        Self::Mcu: McuCreator<Device::StorageConfig>,
    {
    }

    async fn send(&self, _: &[u8]) {}

    async fn receive(&self, _: &mut [u8]) -> usize {
        core::future::pending().await
    }
}
