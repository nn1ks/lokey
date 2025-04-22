use crate::mcu::Mcu;
use crate::{Address, internal};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use embassy_executor::Spawner;

pub struct TransportConfig;

pub struct Transport<M> {
    phantom: PhantomData<M>,
}

impl<M: Mcu> internal::Transport for Transport<M> {
    type Config = TransportConfig;
    type Mcu = M;

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
