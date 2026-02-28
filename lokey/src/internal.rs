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
use derive_more::{Display, Error};
use generic_array::{ArrayLength, GenericArray};

declare_const_for_feature_group!(
    RECEIVER_SLOTS,
    [
        ("internal-receiver-slots-8", 8),
        ("internal-receiver-slots-16", 16),
        ("internal-receiver-slots-24", 24),
        ("internal-receiver-slots-32", 32),
        ("internal-receiver-slots-40", 40),
        ("internal-receiver-slots-48", 48),
        ("internal-receiver-slots-56", 56),
        ("internal-receiver-slots-64", 64),
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, Error)]
#[display("The maximum number of receivers ({}) was reached", RECEIVER_SLOTS)]
pub struct MaximumReceiversReached;

pub type DeviceTransport<D, T> = <T as Transports<<D as Device>::Mcu>>::InternalTransport;

pub trait Message: Send + 'static {
    type Size: ArrayLength;

    const TAG: [u8; 4];

    fn from_bytes(bytes: GenericArray<u8, Self::Size>) -> Option<Self>
    where
        Self: Sized;

    fn to_bytes(&self) -> GenericArray<u8, Self::Size>;
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
