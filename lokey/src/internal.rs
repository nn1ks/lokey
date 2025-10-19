#[cfg(feature = "internal-ble")]
pub mod ble;
mod channel;
pub mod empty;

use crate::mcu::Mcu;
use crate::util::declare_const_for_feature_group;
use crate::{Address, Device, Transports};
pub use channel::{Channel, DynChannelRef, Receiver};
use core::any::Any;
use core::future::Future;
use generic_array::{ArrayLength, GenericArray};

declare_const_for_feature_group!(
    MAX_NUM_RECEIVERS,
    [
        ("max-num-internal-receivers-8", 8),
        ("max-num-internal-receivers-16", 16),
        ("max-num-internal-receivers-24", 24),
        ("max-num-internal-receivers-32", 32),
        ("max-num-internal-receivers-40", 40),
        ("max-num-internal-receivers-48", 48),
        ("max-num-internal-receivers-56", 56),
        ("max-num-internal-receivers-64", 64),
    ]
);

declare_const_for_feature_group!(
    MAX_MESSAGE_SIZE,
    [
        ("max-internal-message-size-8", 8),
        ("max-internal-message-size-16", 16),
        ("max-internal-message-size-32", 32),
        ("max-internal-message-size-64", 64),
        ("max-internal-message-size-128", 128),
        ("max-internal-message-size-256", 256),
        ("max-internal-message-size-512", 512),
        ("max-internal-message-size-1024", 1024),
    ]
);

const MAX_MESSAGE_SIZE_WITH_TAG: usize = MAX_MESSAGE_SIZE + 4;

pub type DeviceTransport<D, T> = <T as Transports<<D as Device>::Mcu>>::InternalTransport;

pub trait Message: Send + 'static {
    type SIZE: ArrayLength;

    const TAG: [u8; 4];

    fn from_bytes(bytes: GenericArray<u8, Self::SIZE>) -> Option<Self>
    where
        Self: Sized;

    fn to_bytes(&self) -> GenericArray<u8, Self::SIZE>;
}

pub trait Transport: Any {
    type Config;
    type Mcu: Mcu;

    fn create(
        config: Self::Config,
        mcu: &'static Self::Mcu,
        address: Address,
    ) -> impl Future<Output = Self>;

    fn run(&self) -> impl Future<Output = ()>;

    fn send(&self, message_bytes: &[u8]) -> impl Future<Output = ()>;

    fn receive(&self, buf: &mut [u8]) -> impl Future<Output = usize>;
}
