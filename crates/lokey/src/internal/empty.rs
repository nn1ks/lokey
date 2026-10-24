//! Empty internal transport implementation.

use crate::{Address, internal};
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

impl<Mcu: 'static> internal::Transport for Transport<Mcu> {
    type Config = TransportConfig;
    type Mcu = Mcu;

    async fn create(_: Self::Config, _: &'static Self::Mcu, _: Address) -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    async fn run<Storage>(&self, _: &'static Storage)
    where
        Storage: crate::storage::Storage,
    {
    }

    async fn send(&self, _: &[u8]) {}

    async fn receive(&self, _: &mut [u8]) -> usize {
        core::future::pending().await
    }
}
