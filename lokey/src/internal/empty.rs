use crate::mcu::Mcu;
use crate::{Address, internal};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use embassy_executor::Spawner;

pub struct TransportConfig;

impl<M: Mcu> internal::TransportConfig<M> for TransportConfig {
    type Transport = Transport;

    async fn init(self, _: &'static M, _: Address, _: Spawner) -> Self::Transport {
        Transport
    }
}

pub struct Transport;

impl internal::Transport for Transport {
    fn send(&self, _message_bytes: &[u8]) {}

    fn receive(&self) -> Pin<Box<dyn Future<Output = Vec<u8>> + '_>> {
        Box::pin(async { core::future::pending().await })
    }
}
