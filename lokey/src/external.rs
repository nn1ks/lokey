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
use crate::util::declare_const_for_feature_group;
use crate::{Address, Device, Transports, internal};
pub use channel::{Channel, DynChannelRef, Receiver};
use core::any::Any;
use core::future::Future;
pub use message_service::MessageServiceRegistry;
pub use r#override::{IdentityOverride, MessageSender, Override};

declare_const_for_feature_group!(
    RECEIVER_SLOTS,
    [
        ("external-receiver-slots-8", 8),
        ("external-receiver-slots-16", 16),
        ("external-receiver-slots-24", 24),
        ("external-receiver-slots-32", 32),
        ("external-receiver-slots-40", 40),
        ("external-receiver-slots-48", 48),
        ("external-receiver-slots-56", 56),
        ("external-receiver-slots-64", 64),
    ]
);

declare_const_for_feature_group!(
    OBSERVER_SLOTS,
    [
        ("external-observer-slots-8", 8),
        ("external-observer-slots-16", 16),
        ("external-observer-slots-24", 24),
        ("external-observer-slots-32", 32),
        ("external-observer-slots-40", 40),
        ("external-observer-slots-48", 48),
        ("external-observer-slots-56", 56),
        ("external-observer-slots-64", 64),
    ]
);

pub struct MaximumReceiversReached;

pub struct MaximumObserversReached;

pub type DeviceTransport<D, T> = <T as Transports<<D as Device>::Mcu>>::ExternalTransport;

pub type DeviceTransportTxMessage<D, T> =
    <<T as Transports<<D as Device>::Mcu>>::ExternalTransport as Transport>::TxMessage;

pub type DeviceTransportRxMessage<D, T> =
    <<T as Transports<<D as Device>::Mcu>>::ExternalTransport as Transport>::RxMessage;

pub trait Message: Any + Clone + Send + Sync {}

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
    type TxMessage: Message + Clone;
    type RxMessage: Message + Clone;

    fn create<T: internal::Transport<Mcu = Self::Mcu>>(
        config: Self::Config,
        mcu: &'static Self::Mcu,
        address: Address,
        internal_channel: &'static internal::Channel<T>,
    ) -> impl Future<Output = Self>;

    fn run(&self) -> impl Future<Output = ()>;

    fn send(&self, message: Self::TxMessage) -> impl Future<Output = ()>;

    fn receive(&self) -> impl Future<Output = Self::RxMessage>;

    /// Activates or deactivates the transport.
    ///
    /// Returns `false` if this transport does not support deactivating, otherwise `true`.
    fn set_active(&self, value: bool) -> impl Future<Output = bool> {
        async {
            let _ = value;
            false
        }
    }

    /// Returns whether the transport is currently activated.
    fn is_active(&self) -> bool {
        true
    }

    fn wait_for_activation_request(&self) -> impl Future<Output = ()> {
        core::future::pending()
    }
}
