use super::Messages0;
use crate::{Address, external, internal, mcu};
use core::marker::PhantomData;

pub struct TransportConfig;

pub struct Transport<Mcu, Messages> {
    phantom: PhantomData<(Mcu, Messages)>,
}

impl<Mcu: mcu::Mcu> external::Transport for Transport<Mcu, Messages0> {
    type Config = TransportConfig;
    type Mcu = Mcu;
    type Messages = Messages0;

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

    fn send(&self, _: Messages0) {}
}
