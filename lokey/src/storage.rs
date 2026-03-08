mod default;
mod empty;

use core::any::Any;
use core::fmt::Debug;
pub use default::DefaultStorage;
pub use empty::{EmptyStorage, EmptyStorageDriver};
use generic_array::{ArrayLength, GenericArray};

pub trait StorageDriver: Any {
    type Storage: Storage;
    type Mcu;
    type Config: Default;
    fn create_storage(mcu: &'static Self::Mcu, config: Self::Config) -> Self::Storage;
}

pub trait Storage: Any {
    type FlashError: Debug;

    fn remove<E: Entry>(
        &self,
        tag_params: E::TagParams,
    ) -> impl Future<Output = Result<(), Error<<Self as Storage>::FlashError>>>;

    fn store<E: Entry>(
        &self,
        tag_params: E::TagParams,
        entry: &E,
    ) -> impl Future<Output = Result<(), Error<<Self as Storage>::FlashError>>>;

    fn fetch<E: Entry>(
        &self,
        tag_params: E::TagParams,
    ) -> impl Future<Output = Result<Option<E>, Error<<Self as Storage>::FlashError>>>;
}

pub const ENTRY_TAG_SIZE: usize = 8;

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

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<E> {
    Flash(E),
    FullStorage,
    Corrupted,
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
