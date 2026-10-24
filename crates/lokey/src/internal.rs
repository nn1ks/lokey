//! Internal communication between components and devices.

mod channel;
pub mod empty;

use crate::util::declare_const_for_feature_group;
use crate::{Address, Device, Transports};
pub use channel::{Channel, DynChannelRef, Receiver};
use core::any::Any;
use core::future::Future;
use derive_more::{Display, Error};
use generic_array::{ArrayLength, GenericArray};

declare_const_for_feature_group!(
    /// The maximum number of receivers for the internal channel.
    ///
    /// This can be configured via the `internal-receiver-slots-*` features. See the
    /// [crate-level documentation](crate) for details.
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
    /// The maximum size of an internal message.
    ///
    /// This can be configured via the `max-internal-message-size-*` features. See the
    /// [crate-level documentation](crate) for details.
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

/// The size of the tag used to identify message types in internal transports.
pub const MESSAGE_TAG_SIZE: usize = 4;

/// The maximum total size of an internal message, including the tag.
pub const MAX_MESSAGE_SIZE_WITH_TAG: usize = MAX_MESSAGE_SIZE + MESSAGE_TAG_SIZE;

/// Error indicating that the maximum number of receivers for the internal channel has been reached.
///
/// This error is returned when attempting to create a new receiver for the internal channel with
/// [`Channel::receiver`], but the maximum number of allowed receivers (as determined by the
/// `internal-receiver-slots-*` feature) is already reached.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[display("The maximum number of receivers ({}) was reached", RECEIVER_SLOTS)]
pub struct MaximumReceiversReached;

/// Type alias for the internal transport used by a device.
pub type DeviceTransport<D, T> = <T as Transports<<D as Device>::Mcu>>::InternalTransport;

/// Trait for messages that can be sent through internal transports.
pub trait Message: Send + 'static {
    /// The length of the byte array that this message is serialized to, excluding the tag.
    type Size: ArrayLength;

    /// The tag bytes used to identify this message type in the transport.
    const TAG: [u8; MESSAGE_TAG_SIZE];

    /// Deserializes the message from the specified bytes.
    ///
    /// The tag bytes are not included in the provided byte array.
    fn from_bytes(bytes: GenericArray<u8, Self::Size>) -> Option<Self>
    where
        Self: Sized;

    /// Serializes this message to a byte array.
    ///
    /// The tag bytes are not included in the returned byte array.
    fn to_bytes(&self) -> GenericArray<u8, Self::Size>;
}

/// Trait for exchanging messages between devices in a multi-part device setup.
pub trait Transport: Any {
    /// The configuration for this transport.
    type Config;

    /// The MCU type that this transport runs on.
    type Mcu: 'static;

    /// Creates and initializes the transport instance.
    fn create(
        config: Self::Config,
        mcu: &'static Self::Mcu,
        address: Address,
    ) -> impl Future<Output = Self>;

    /// Runs the transport background task.
    ///
    /// This drives ongoing transport work such as receiving incoming message bytes and handling
    /// transport-specific processing.
    fn run<Storage>(&self, storage: &'static Storage) -> impl Future<Output = ()>
    where
        Storage: crate::storage::Storage;

    /// Sends a message through this transport.
    fn send(&self, message_bytes: &[u8]) -> impl Future<Output = ()>;

    /// Receives a message from this transport.
    ///
    /// The message bytes are written to the provided buffer, and the number of bytes received is
    /// returned.
    fn receive(&self, buf: &mut [u8]) -> impl Future<Output = usize>;
}
