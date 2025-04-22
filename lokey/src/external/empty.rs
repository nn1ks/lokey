use crate::external::{self, Messages};
use crate::mcu::Mcu;
use crate::{Address, internal};
use core::marker::PhantomData;
use embassy_executor::Spawner;

pub struct TransportConfig;

pub struct Transport<M, T> {
    phantom: PhantomData<(M, T)>,
}

impl<M: Mcu, T: Messages> external::Transport for Transport<M, T> {
    type Config = TransportConfig;
    type Mcu = M;
    type Messages = T;

    async fn create(
        _: Self::Config,
        _: &'static Self::Mcu,
        _: Address,
        _: Spawner,
        _: internal::DynChannel,
    ) -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    async fn run(&self) {}

    fn send(&self, _: T) {}
}
