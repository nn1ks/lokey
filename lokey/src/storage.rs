//! Persistent storage for device data.
//!
//! This module provides abstractions for storing and retrieving typed entries in non-volatile
//! memory.
//!
//! Main building blocks:
//! - [`StorageDriver`]: Creates a concrete storage backend from MCU-specific resources.
//! - [`Storage`]: Async API to store, fetch, and remove typed entries.
//! - [`Entry`]: Defines how a type is tagged and serialized for storage.
//! - [`Error`]: Common storage error type used across backends.
//!
//! Entries are identified by an 8-byte tag (see [`ENTRY_TAG_SIZE`]). Tags can be parameterized via
//! [`Entry::TagParams`] to support multiple stored instances of the same entry type.

mod default;
mod empty;

use core::any::Any;
use core::fmt::Debug;
pub use default::DefaultStorage;
pub use empty::{EmptyStorage, EmptyStorageDriver};
use generic_array::{ArrayLength, GenericArray};

/// Trait to create and configure a storage instance for a specific MCU.
pub trait StorageDriver: Any {
    /// The type of the storage created by this driver.
    type Storage: Storage;

    /// The type of the MCU that this storage driver is designed for.
    type Mcu;

    /// The configuration type for this storage driver.
    type Config: Default;

    /// Creates a storage instance for the specified MCU and configuration.
    fn create_storage(mcu: &'static Self::Mcu, config: Self::Config) -> Self::Storage;
}

/// Trait for persistent storage of typed entries.
pub trait Storage: Any {
    /// The error type returned by flash operations in this storage.
    type FlashError: Debug;

    /// Removes the specified entry from storage.
    fn remove<E: Entry>(
        &self,
        tag_params: E::TagParams,
    ) -> impl Future<Output = Result<(), Error<<Self as Storage>::FlashError>>>;

    /// Stores the specified entry in storage.
    fn store<E: Entry>(
        &self,
        tag_params: E::TagParams,
        entry: &E,
    ) -> impl Future<Output = Result<(), Error<<Self as Storage>::FlashError>>>;

    /// Fetches the specified entry from storage.
    fn fetch<E: Entry>(
        &self,
        tag_params: E::TagParams,
    ) -> impl Future<Output = Result<Option<E>, Error<<Self as Storage>::FlashError>>>;
}

/// The size of the tag used to identify entries in storage.
pub const ENTRY_TAG_SIZE: usize = 8;

/// Trait for types that can be stored in persistent storage.
pub trait Entry {
    /// The length of the byte array that this type is serialized to.
    type Size: ArrayLength;

    /// The type of the parameter that is passed to the [`tag`](Self::tag) function.
    ///
    /// By settings this to a type that can have different values, multiple instances of the entry
    /// can be stored. If you only ever need to store one instance of this entry type, the type can
    /// be set to `()`.
    type TagParams;

    /// Creates a unique tag to identify this entry type.
    fn tag(params: Self::TagParams) -> [u8; ENTRY_TAG_SIZE];

    /// Deserializes the entry from the specified bytes.
    ///
    /// Returns [`None`] if the entry can not be deserialized.
    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self>
    where
        Self: Sized;

    /// Serializes this entry to a byte array.
    fn to_bytes(&self) -> GenericArray<u8, Self::Size>;
}

/// Error type for storage operations.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<E> {
    /// Flash error returned by the underlying flash storage.
    Flash(E),
    /// The storage is full and can not store any more entries.
    FullStorage,
    /// The storage is corrupted and can not be used.
    Corrupted,
    /// The entry is too big to be stored in the storage.
    EntryTooBig,
}

impl<E> Error<E> {
    fn from_sequential_storage(error: sequential_storage::Error<E>) -> Self {
        match error {
            sequential_storage::Error::Storage { value } => Self::Flash(value),
            sequential_storage::Error::FullStorage => Self::FullStorage,
            sequential_storage::Error::Corrupted {} => Self::Corrupted,
            sequential_storage::Error::BufferTooBig => {
                // Should not be possible because the buffer is always created with the correct size
                // in the methods of the `Storage` type
                panic!("Unexpected storage error: BufferTooBig");
            }
            sequential_storage::Error::BufferTooSmall(v) => {
                // Should not be possible because the buffer is always created with the correct size
                // in the methods of the `Storage` type
                panic!("Unexpected storage error: BufferTooSmall({})", v);
            }
            sequential_storage::Error::SerializationError(v) => {
                // Should not be possible because a byte array is always used as the value which
                // doesn't return serialization errors
                panic!("Unexpected storage error: SerializationError({})", v);
            }
            sequential_storage::Error::ItemTooBig => Self::EntryTooBig,
            _ => {
                // The `sequential_storage::Error` type is marked as non-exhaustive so we have to
                // handle this case as well in case a new error variant is added. At the moment all
                // variants are handled so we can just panic, but this might have to be changed in
                // the future.
                panic!("Unexpected storage error: Other");
            }
        }
    }
}
