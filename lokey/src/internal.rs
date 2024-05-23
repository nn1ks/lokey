#[cfg(feature = "ble")]
pub mod ble;
mod channel;
pub mod empty;

pub use channel::{Channel, DynChannel, Receiver};

use crate::{mcu::Mcu, Device};
use alloc::{boxed::Box, vec::Vec};
use core::{any::Any, future::Future, pin::Pin};
use embassy_executor::Spawner;

pub type DeviceChannel<D> =
    <<D as Device>::InternalChannelConfig as ChannelConfig<<D as Device>::Mcu>>::Channel;

pub trait MessageTag {
    const TAG: [u8; 4];
}

pub trait Message: Send + 'static {
    fn from_bytes(bytes: &[u8]) -> Option<Self>
    where
        Self: Sized;

    fn to_bytes(&self) -> Vec<u8>;
}

pub trait ChannelConfig<M: Mcu> {
    type Channel: ChannelImpl;
    fn init(self, mcu: &'static M, spawner: Spawner) -> impl Future<Output = Self::Channel>;
}

pub trait ChannelImpl: Any {
    fn send(&self, message_bytes: &[u8]) -> Pin<Box<dyn Future<Output = ()>>>;
    fn receive(&self) -> Pin<Box<dyn Future<Output = Vec<u8>>>>;
}
