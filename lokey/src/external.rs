#[cfg(feature = "external-ble")]
pub mod ble;
mod channel;
pub mod empty;
mod message_service;
mod r#override;
pub mod toggle;
#[cfg(feature = "external-usb")]
pub mod usb;
#[cfg(all(feature = "external-usb", feature = "external-ble"))]
pub mod usb_ble;

use crate::mcu::Mcu;
use crate::{Address, Device, Transports, internal};
use alloc::boxed::Box;
pub use channel::{Channel, DynChannelRef, Receiver};
use core::any::Any;
use core::future::Future;
use core::mem::transmute;
use core::pin::Pin;
use dyn_clone::DynClone;
pub use message_service::MessageServiceRegistry;
pub use r#override::{MessageSender, Override};

pub type DeviceTransport<D, T> = <T as Transports<<D as Device>::Mcu>>::ExternalTransport;

pub trait Message: Any + DynClone + Send + Sync {}

dyn_clone::clone_trait_object!(Message);

pub trait TryFromMessage<T>: Sized {
    fn try_from_message(value: T) -> Result<Self, MismatchedMessageType>;
}

impl<T: Message> TryFromMessage<T> for T {
    fn try_from_message(value: T) -> Result<Self, MismatchedMessageType> {
        Ok(value)
    }
}

#[derive(Debug, Clone)]
pub enum NoMessage {}

impl Message for NoMessage {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnsupportedMessageType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MismatchedMessageType;

pub trait Transport: Any {
    type Config;
    type Mcu: Mcu;
    type TxMessage: Message;
    type RxMessage: Message;

    fn create<T: internal::Transport<Mcu = Self::Mcu>>(
        config: Self::Config,
        mcu: &'static Self::Mcu,
        address: Address,
        internal_channel: &'static internal::Channel<T>,
    ) -> impl Future<Output = Self>;

    fn run(&self) -> impl Future<Output = ()>;

    fn send(&self, message: Self::TxMessage);

    fn receive(&self) -> Pin<Box<dyn Future<Output = Self::RxMessage> + '_>>;

    /// Activates or deactivates the transport.
    ///
    /// Returns `false` if this transport does not support deactivating, otherwise `true`.
    fn set_active(&self, value: bool) -> bool {
        let _ = value;
        false
    }

    /// Returns whether the transport is currently activated.
    fn is_active(&self) -> bool {
        true
    }

    fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(core::future::pending())
    }
}

trait DynTransportTrait: Any {
    fn try_send_dyn(&self, message: Box<dyn Message>) -> Result<(), UnsupportedMessageType>;
    fn set_active(&self, value: bool) -> bool;
    fn is_active(&self) -> bool;
    fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>>;
}

impl<T: Transport> DynTransportTrait for T {
    fn try_send_dyn(&self, message: Box<dyn Message>) -> Result<(), UnsupportedMessageType> {
        let message: Box<dyn Any> = message;
        let message = message
            .downcast::<T::TxMessage>()
            .map_err(|_| UnsupportedMessageType)?;
        Transport::send(self, *message);
        Ok(())
    }

    fn set_active(&self, value: bool) -> bool {
        Transport::set_active(self, value)
    }

    fn is_active(&self) -> bool {
        Transport::is_active(self)
    }

    fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Transport::wait_for_activation_request(self)
    }
}

#[repr(transparent)]
pub struct DynTransport(dyn DynTransportTrait);

impl DynTransport {
    pub const fn from_ref<T: Transport>(value: &T) -> &Self {
        let value: &dyn DynTransportTrait = value;
        unsafe { transmute(value) }
    }

    pub fn try_send_dyn(&self, message: Box<dyn Message>) -> Result<(), UnsupportedMessageType> {
        self.0.try_send_dyn(message)
    }

    pub fn set_active(&self, value: bool) -> bool {
        self.0.set_active(value)
    }

    pub fn is_active(&self) -> bool {
        self.0.is_active()
    }

    pub fn wait_for_activation_request(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        self.0.wait_for_activation_request()
    }
}
