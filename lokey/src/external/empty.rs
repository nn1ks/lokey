use super::Messages0;
use crate::mcu::Mcu;
use crate::{Address, external, internal};
use core::marker::PhantomData;
use embassy_executor::Spawner;

pub struct TransportConfig;

pub struct Transport<M, T> {
    phantom: PhantomData<(M, T)>,
}

impl<M: Mcu> external::Transport for Transport<M, Messages0> {
    type Config = TransportConfig;
    type Mcu = M;
    type Messages = Messages0;

    async fn create(
        _: Self::Config,
        _: &'static Self::Mcu,
        _: Address,
        _: Spawner,
        _: internal::DynChannelRef<'static>,
    ) -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    async fn run(&self) {}

    fn send(&self, _: Messages0) {}
}
