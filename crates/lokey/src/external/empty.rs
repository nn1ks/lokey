//! Empty external transport implementation.

use crate::external::NoMessage;
use crate::{Address, AnyState, Context, McuCreator, external, internal, storage};
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

    async fn create<Storage, InternalTransport>(
        _: Self::Config,
        _: Address,
        _: &'static Self::Mcu,
        _: &'static Storage,
        _: &'static internal::Channel<InternalTransport>,
    ) -> Self
    where
        Storage: storage::Storage,
        InternalTransport: internal::Transport<Mcu = Self::Mcu>,
    {
        Self {
            phantom: PhantomData,
        }
    }

    async fn run<Device, Transports, State>(&self, _: Context<Device, Transports, State>)
    where
        Device: crate::Device<Mcu = Self::Mcu>,
        Transports: crate::Transports<Self::Mcu>,
        State: AnyState,
        Self::Mcu: McuCreator<Device::StorageConfig>,
    {
    }

    async fn send(&self, _: Self::TxMessage) {}

    async fn receive(&self) -> Self::RxMessage {
        core::future::pending().await
    }
}
