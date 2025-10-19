use crate::{Address, internal, mcu};
use core::future::Future;
use core::marker::PhantomData;

pub struct TransportConfig;

pub struct Transport<Mcu> {
    phantom: PhantomData<Mcu>,
}

impl<Mcu: mcu::Mcu> internal::Transport for Transport<Mcu> {
    type Config = TransportConfig;
    type Mcu = Mcu;

    async fn create(_: Self::Config, _: &'static Self::Mcu, _: Address) -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    async fn run(&self) {}

    fn send(&self, _: &[u8]) {}

    fn receive(&self, _: &mut [u8]) -> impl Future<Output = usize> {
        core::future::pending()
    }
}
