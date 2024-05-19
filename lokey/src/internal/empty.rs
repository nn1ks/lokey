use crate::internal;
use crate::mcu::Mcu;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::future::Future;
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
    fn send(&self, _message_bytes: &[u8]) -> Box<dyn Future<Output = ()>> {
        Box::new(async {})
    }

    fn receive(&self) -> Box<dyn Future<Output = Vec<u8>>> {
        Box::new(async { core::future::pending().await })
    }
}
