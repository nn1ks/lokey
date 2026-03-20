//! External communication with a host.

mod channel;
pub mod empty;
mod r#override;
pub mod toggle;

use crate::util::declare_const_for_feature_group;
use crate::{Address, Device, Transports, internal};
pub use channel::{Channel, DynChannelRef, Receiver};
use core::any::Any;
use core::future::Future;
use derive_more::{Display, Error, From};
/// Derive macro for [`external::Message`](crate::external::Message).
///
/// This macro generates an implementation of the [`Message`] trait for structs and enums.
///
/// - **Structs** are treated as standalone message types (no inner message types).
/// - **Enums** are treated as wrapper message types, where each variant must have exactly one
///   unnamed field (tuple variant).
///
/// # Example
///
/// ```
/// use lokey::external::Message;
///
/// #[derive(Clone, Message)]
/// pub struct KeyboardEvent;
///
/// #[derive(Clone, Message)]
/// pub struct MouseEvent;
///
/// #[derive(Clone, Message)]
/// pub enum DeviceEvent {
///     Keyboard(KeyboardEvent),
///     Mouse(MouseEvent),
/// }
/// ```
#[cfg(feature = "macros")]
pub use lokey_macros::ExternalMessage as Message;
pub use r#override::{IdentityOverride, MessageSender, Override};

declare_const_for_feature_group!(
    /// The maximum number of receivers for the external channel.
    ///
    /// This can be configured via the `external-receiver-slots-*` features. See the
    /// [crate-level documentation](crate) for details.
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
    /// The maximum number of observers for the external channel.
    ///
    /// This can be configured via the `external-observer-slots-*` features. See the
    /// [crate-level documentation](crate) for details.
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

/// Error indicating that the provided message type is not supported by the transport.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[display("The provided message type is not supported")]
pub struct UnsupportedMessageType;

/// Error indicating that the message type does not match the expected type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[display("The message type does not match")]
pub struct MismatchedMessageType;

/// Error indicating that the maximum number of receivers for the external channel has been reached.
///
/// This error is returned when attempting to create a new receiver for the external channel with
/// [`Channel::receiver`] or [`Channel::try_receiver`], but the maximum number of allowed receivers
/// (as determined by the `external-receiver-slots-*` feature) is already reached.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[display("The maximum number of receivers ({}) was reached", RECEIVER_SLOTS)]
pub struct MaximumReceiversReached;

/// Error indicating that the maximum number of observers for the external channel has been reached.
///
/// This error is returned when attempting to create a new observer for the external channel with
/// [`Channel::observer`] or [`Channel::try_observer`], but the maximum number of allowed observers
/// (as determined by the `external-observer-slots-*` feature) is already reached.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[display("The maximum number of observers ({}) was reached", OBSERVER_SLOTS)]
pub struct MaximumObserversReached;

/// Error indicating that a receiver with [`Channel::try_receiver`] could not be created.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, Error, From)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TryReceiverError {
    /// The provided message type is not supported by the transport.
    UnsupportedMessageType(UnsupportedMessageType),
    /// The maximum number of receivers for the external channel has been reached.
    MaximumReceiversReached(MaximumReceiversReached),
}

/// Error indicating that an observer with [`Channel::try_observer`] could not be created.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, Error, From)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TryObserverError {
    /// The provided message type is not supported by the transport.
    UnsupportedMessageType(UnsupportedMessageType),
    /// The maximum number of observers for the external channel has been reached.
    MaximumObserversReached(MaximumObserversReached),
}

/// Type alias for the external transport used by a device.
pub type DeviceTransport<D, T> = <T as Transports<<D as Device>::Mcu>>::ExternalTransport;

/// Type alias for the message type sent by the device's external transport.
pub type DeviceTransportTxMessage<D, T> =
    <<T as Transports<<D as Device>::Mcu>>::ExternalTransport as Transport>::TxMessage;

/// Type alias for the message type received by the device's external transport.
pub type DeviceTransportRxMessage<D, T> =
    <<T as Transports<<D as Device>::Mcu>>::ExternalTransport as Transport>::RxMessage;

/// Trait for messages that can be sent through external transports.
pub trait Message: Any + Clone + Send + Sync {
    /// Returns whether this message can contain inner messages of type `M`.
    fn has_inner_message<M: Message>() -> bool;

    /// If this message contains an inner message of type `M`, returns a reference to it. Otherwise,
    /// returns `None`.
    fn inner_message<M: Message>(&self) -> Option<&M>;

    /// Tries to convert this message from an inner message of type `M` to this message type. Returns an
    /// error if this message can not contain inner messages of type `M`.
    fn try_from_inner_message(value: &dyn Any) -> Result<Self, MismatchedMessageType>
    where
        Self: Sized;
}

/// Conversion trait for transforming one external message type into another.
///
/// This is used by the external channel APIs when a caller requests a specific message type that
/// may differ from the transport message type.
///
/// `T` is the source message type, and `Self` is the target message type.
pub trait TryFromMessage<T>: Sized {
    /// Tries to convert the provided message of type `T` to this message type. Returns an error if
    /// this message type can not be converted from messages of type `T`.
    fn try_from_message(value: T) -> Result<Self, MismatchedMessageType>;
}

impl<T: Message> TryFromMessage<T> for T {
    /// Identity conversion for equal source and target message types.
    fn try_from_message(value: T) -> Result<Self, MismatchedMessageType> {
        Ok(value)
    }
}

/// Message type representing that no messages are supported.
///
/// This uninhabited enum is useful for transports that never send or receive external messages.
/// Because it has no variants, no value of this type can exist.
#[derive(Debug, Clone)]
pub enum NoMessage {}

impl Message for NoMessage {
    fn has_inner_message<M: Message>() -> bool {
        false
    }

    fn inner_message<M: Message>(&self) -> Option<&M> {
        None
    }

    fn try_from_inner_message(_: &dyn Any) -> Result<Self, MismatchedMessageType>
    where
        Self: Sized,
    {
        Err(MismatchedMessageType)
    }
}

/// Trait for external transports used by devices to communicate with a host.
pub trait Transport: Any {
    /// The configuration for this transport.
    type Config;

    /// The MCU type that this transport runs on.
    type Mcu: 'static;

    /// The message type sent by this transport.
    type TxMessage: Message + Clone;

    /// The message type received by this transport.
    type RxMessage: Message + Clone;

    /// Creates and initializes the transport instance.
    fn create<T>(
        config: Self::Config,
        mcu: &'static Self::Mcu,
        address: Address,
        internal_channel: &'static internal::Channel<T>,
    ) -> impl Future<Output = Self>
    where
        T: internal::Transport<Mcu = Self::Mcu>;

    /// Runs the transport background task.
    ///
    /// This drives ongoing transport work such as receiving incoming message bytes and handling
    /// transport-specific processing.
    fn run<Storage>(&self, storage: &'static Storage) -> impl Future<Output = ()>
    where
        Storage: crate::storage::Storage;

    /// Sends a message through this transport.
    fn send(&self, message: Self::TxMessage) -> impl Future<Output = ()>;

    /// Receives a message from this transport.
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

    /// Waits for an activation request from the host.
    fn wait_for_activation_request(&self) -> impl Future<Output = ()> {
        core::future::pending()
    }
}
