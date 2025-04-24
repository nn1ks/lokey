use crate::{Address, internal, mcu};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use embassy_executor::Spawner;

pub struct TransportConfig;

pub struct Transport<Mcu> {
    phantom: PhantomData<Mcu>,
}

impl<Mcu: mcu::Mcu> internal::Transport for Transport<Mcu> {
    type Config = TransportConfig;
    type Mcu = Mcu;

    async fn create(_: Self::Config, _: &'static Self::Mcu, _: Address, _: Spawner) -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    async fn run(&self) {}

    fn send(&self, _message_bytes: &[u8]) {}

    fn receive(&self) -> Pin<Box<dyn Future<Output = Vec<u8>> + '_>> {
        Box::pin(async { core::future::pending().await })
    }
}
