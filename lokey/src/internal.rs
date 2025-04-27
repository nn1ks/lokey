#[cfg(feature = "internal-ble")]
pub mod ble;
mod channel;
pub mod empty;

use crate::mcu::Mcu;
use crate::{Address, Device, Transports};
use alloc::boxed::Box;
use alloc::vec::Vec;
pub use channel::{Channel, DynChannelRef, Receiver};
use core::any::Any;
use core::future::Future;
use core::mem::transmute;
use core::pin::Pin;

pub type DeviceTransport<D, T> = <T as Transports<<D as Device>::Mcu>>::InternalTransport;

pub trait Message: Send + 'static {
    type Bytes: for<'a> TryFrom<&'a [u8]> + Into<Vec<u8>>;

    const TAG: [u8; 4];

    fn from_bytes(bytes: &Self::Bytes) -> Option<Self>
    where
        Self: Sized;

    fn to_bytes(&self) -> Self::Bytes;
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

    fn receive(&self) -> Pin<Box<dyn Future<Output = Vec<u8>> + '_>>;
}

trait DynTransportTrait: Any {
    fn send(&self, message_bytes: &[u8]);
    fn receive(&self) -> Pin<Box<dyn Future<Output = Vec<u8>> + '_>>;
}

impl<T: Transport> DynTransportTrait for T {
    fn send(&self, message_bytes: &[u8]) {
        Transport::send(self, message_bytes)
    }

    fn receive(&self) -> Pin<Box<dyn Future<Output = Vec<u8>> + '_>> {
        Transport::receive(self)
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

    pub fn receive(&self) -> Pin<Box<dyn Future<Output = Vec<u8>> + '_>> {
        self.0.receive()
    }
}
