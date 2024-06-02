use crate::{internal, mcu::Mcu};
use alloc::{boxed::Box, vec::Vec};
use core::{future::Future, pin::Pin};
use embassy_executor::Spawner;

pub struct ChannelConfig;

impl<M: Mcu> internal::ChannelConfig<M> for ChannelConfig {
    type Channel = Channel;

    async fn init(self, _: &'static M, _: Spawner) -> Self::Channel {
        Channel
    }
}

pub struct Channel;

impl internal::ChannelImpl for Channel {
    fn send(&self, _message_bytes: &[u8]) {}

    fn receive(&self) -> Pin<Box<dyn Future<Output = Vec<u8>> + '_>> {
        Box::pin(async { core::future::pending().await })
    }
}
