use crate::external::{self, Message};
use crate::internal;
use crate::mcu::Mcu;
use embassy_executor::Spawner;

pub struct ChannelConfig;

impl<M: Mcu> external::ChannelConfig<M> for ChannelConfig {
    type Channel = Channel;

    async fn init(self, _: &'static M, _: Spawner, _: internal::DynChannel) -> Self::Channel {
        Channel
    }
}

pub struct Channel;

impl external::ChannelImpl for Channel {
    fn send(&self, _: Message) {}
}
