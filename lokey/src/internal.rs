#[cfg(feature = "internal-ble")]
pub mod ble;
mod channel;
pub mod empty;

use crate::mcu::Mcu;
use crate::{Address, Device, Transports};
pub use channel::{Channel, DynChannelRef, Receiver};
use core::any::Any;
use core::future::Future;
use core::mem::transmute;
use generic_array::{ArrayLength, GenericArray};

#[cfg(all(
    not(feature = "max-internal-message-size-8"),
    not(feature = "max-internal-message-size-16"),
    not(feature = "max-internal-message-size-32"),
    not(feature = "max-internal-message-size-64"),
    not(feature = "max-internal-message-size-128"),
    not(feature = "max-internal-message-size-256"),
    not(feature = "max-internal-message-size-512"),
    not(feature = "max-internal-message-size-1024"),
))]
pub const MAX_MESSAGE_SIZE: usize = 0;

#[cfg(all(
    feature = "max-internal-message-size-8",
    not(feature = "max-internal-message-size-16"),
    not(feature = "max-internal-message-size-32"),
    not(feature = "max-internal-message-size-64"),
    not(feature = "max-internal-message-size-128"),
    not(feature = "max-internal-message-size-256"),
    not(feature = "max-internal-message-size-512"),
    not(feature = "max-internal-message-size-1024"),
))]
pub const MAX_MESSAGE_SIZE: usize = 8;

#[cfg(all(
    feature = "max-internal-message-size-16",
    not(feature = "max-internal-message-size-32"),
    not(feature = "max-internal-message-size-64"),
    not(feature = "max-internal-message-size-128"),
    not(feature = "max-internal-message-size-256"),
    not(feature = "max-internal-message-size-512"),
    not(feature = "max-internal-message-size-1024"),
))]
pub const MAX_MESSAGE_SIZE: usize = 16;

#[cfg(all(
    feature = "max-internal-message-size-32",
    not(feature = "max-internal-message-size-64"),
    not(feature = "max-internal-message-size-128"),
    not(feature = "max-internal-message-size-256"),
    not(feature = "max-internal-message-size-512"),
    not(feature = "max-internal-message-size-1024"),
))]
pub const MAX_MESSAGE_SIZE: usize = 32;

#[cfg(all(
    feature = "max-internal-message-size-64",
    not(feature = "max-internal-message-size-128"),
    not(feature = "max-internal-message-size-256"),
    not(feature = "max-internal-message-size-512"),
    not(feature = "max-internal-message-size-1024"),
))]
pub const MAX_MESSAGE_SIZE: usize = 64;

#[cfg(all(
    feature = "max-internal-message-size-128",
    not(feature = "max-internal-message-size-256"),
    not(feature = "max-internal-message-size-512"),
    not(feature = "max-internal-message-size-1024"),
))]
pub const MAX_MESSAGE_SIZE: usize = 128;

#[cfg(all(
    feature = "max-internal-message-size-256",
    not(feature = "max-internal-message-size-512"),
    not(feature = "max-internal-message-size-1024"),
))]
pub const MAX_MESSAGE_SIZE: usize = 256;

#[cfg(all(
    feature = "max-internal-message-size-512",
    not(feature = "max-internal-message-size-1024"),
))]
pub const MAX_MESSAGE_SIZE: usize = 512;

#[cfg(feature = "max-internal-message-size-1024")]
pub const MAX_MESSAGE_SIZE: usize = 1024;

pub const MAX_MESSAGE_SIZE_WITH_TAG: usize = MAX_MESSAGE_SIZE + 4;

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

    fn send(&self, message_bytes: &[u8]);

    fn receive(&self, buf: &mut [u8]) -> impl Future<Output = usize>;
}

trait DynTransportTrait: Any {
    fn send(&self, message_bytes: &[u8]);
}

impl<T: Transport> DynTransportTrait for T {
    fn send(&self, message_bytes: &[u8]) {
        Transport::send(self, message_bytes)
    }
}

#[repr(transparent)]
pub struct DynTransport(dyn DynTransportTrait);

impl DynTransport {
    pub const fn from_ref<T: Transport>(value: &T) -> &Self {
        let value: &dyn DynTransportTrait = value;
        unsafe { transmute(value) }
    }

    pub fn send(&self, message_bytes: &[u8]) {
        self.0.send(message_bytes)
    }
}
