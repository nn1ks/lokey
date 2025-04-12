use crate::external::{self, Messages};
use crate::mcu::Mcu;
use crate::{Address, internal};
use core::marker::PhantomData;
use embassy_executor::Spawner;

pub struct TransportConfig;

impl<M: Mcu, T: Messages> external::TransportConfig<M, T> for TransportConfig {
    type Transport = Transport<T>;

    async fn init(
        self,
        _: &'static M,
        _: Address,
        _: Spawner,
        _: internal::DynChannel,
    ) -> Self::Transport {
        Transport {
            phantom: PhantomData,
        }
    }
}

pub struct Transport<M> {
    phantom: PhantomData<M>,
}

impl<M: Messages> external::Transport for Transport<M> {
    type Messages = M;

    fn send(&self, _: M) {}
}
