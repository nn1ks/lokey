use crate::external::NoMessage;
use crate::{Address, external, internal, mcu};
use core::marker::PhantomData;

pub struct TransportConfig;

pub struct Transport<Mcu> {
    phantom: PhantomData<Mcu>,
}

impl<Mcu: mcu::Mcu> external::Transport for Transport<Mcu> {
    type Config = TransportConfig;
    type Mcu = Mcu;
    type TxMessage = NoMessage;
    type RxMessage = NoMessage;

    async fn create<T: internal::Transport<Mcu = Self::Mcu>>(
        _: Self::Config,
        _: &'static Self::Mcu,
        _: Address,
        _: &'static internal::Channel<T>,
    ) -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    async fn run(&self) {}

    async fn send(&self, _: Self::TxMessage) {}

    async fn receive(&self) -> Self::RxMessage {
        core::future::pending().await
    }
}
