#[cfg(feature = "ble")]
pub mod ble;
mod channel;
pub mod empty;

use crate::mcu::Mcu;
use crate::{Address, Device, Transports};
use alloc::boxed::Box;
use alloc::vec::Vec;
pub use channel::{Channel, DynChannel, Receiver};
use core::any::Any;
use core::future::Future;
use core::pin::Pin;
use embassy_executor::Spawner;

pub type DeviceTransport<D, T> =
    <<T as Transports<<D as Device>::Mcu>>::InternalTransportConfig as TransportConfig<
        <D as Device>::Mcu,
    >>::Transport;

pub trait Message: Send + 'static {
    type Bytes: for<'a> TryFrom<&'a [u8]> + Into<Vec<u8>>;

    const TAG: [u8; 4];

    fn from_bytes(bytes: &Self::Bytes) -> Option<Self>
    where
        Self: Sized;

    fn to_bytes(&self) -> Self::Bytes;
}

pub trait TransportConfig<M: Mcu> {
    type Transport: Transport;
    fn init(
        self,
        mcu: &'static M,
        address: Address,
        spawner: Spawner,
    ) -> impl Future<Output = Self::Transport>;
}

pub trait Transport: Any {
    fn send(&self, message_bytes: &[u8]);
    fn receive(&self) -> Pin<Box<dyn Future<Output = Vec<u8>> + '_>>;
}
