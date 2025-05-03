use super::{RxMessages0, TxMessages0};
use crate::{Address, external, internal, mcu};
use alloc::boxed::Box;
use core::marker::PhantomData;
use core::pin::Pin;

pub struct TransportConfig;

pub struct Transport<Mcu> {
    phantom: PhantomData<Mcu>,
}

impl<Mcu: mcu::Mcu> external::Transport for Transport<Mcu> {
    type Config = TransportConfig;
    type Mcu = Mcu;
    type TxMessages = TxMessages0;
    type RxMessages = RxMessages0;

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

    fn send(&self, _: TxMessages0) {}

    fn receive(&self) -> Pin<Box<dyn Future<Output = Self::RxMessages>>> {
        Box::pin(core::future::pending())
    }
}
