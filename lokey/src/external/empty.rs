use crate::external::{self, Message};
use crate::mcu::Mcu;
use crate::{Address, internal};
use embassy_executor::Spawner;

pub struct TransportConfig;

impl<M: Mcu> external::TransportConfig<M> for TransportConfig {
    type Transport = Transport;

    async fn init(
        self,
        _: &'static M,
        _: Address,
        _: Spawner,
        _: internal::DynChannel,
    ) -> Self::Transport {
        Transport
    }
}

pub struct Transport;

impl external::Transport for Transport {
    fn send(&self, _: Message) {}
}
