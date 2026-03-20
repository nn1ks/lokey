//! Empty external transport implementation.

use crate::external::NoMessage;
use crate::{Address, external, internal};
use core::marker::PhantomData;

/// Configuration for the empty external transport.
pub struct TransportConfig;

/// An external transport implementation that does not actually send or receive any messages.
///
/// This transport is intended for devices that do not communicate with a host.
pub struct Transport<Mcu> {
    phantom: PhantomData<Mcu>,
}

impl<Mcu> external::Transport for Transport<Mcu>
where
    Mcu: 'static,
{
    type Config = TransportConfig;
    type Mcu = Mcu;
    type TxMessage = NoMessage;
    type RxMessage = NoMessage;

    async fn create<T>(
        _: Self::Config,
        _: &'static Self::Mcu,
        _: Address,
        _: &'static internal::Channel<T>,
    ) -> Self
    where
        T: internal::Transport<Mcu = Self::Mcu>,
    {
        Self {
            phantom: PhantomData,
        }
    }

    async fn run<Storage>(&self, _: &'static Storage)
    where
        Storage: crate::storage::Storage,
    {
    }

    async fn send(&self, _: Self::TxMessage) {}

    async fn receive(&self) -> Self::RxMessage {
        core::future::pending().await
    }
}
