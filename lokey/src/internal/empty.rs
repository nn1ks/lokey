use crate::{Address, internal};
use core::marker::PhantomData;

pub struct TransportConfig;

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
