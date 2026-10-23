//! Persistent storage for device data.
//!
//! This module provides abstractions for storing and retrieving typed entries in non-volatile
//! memory.
//!
//! Main building blocks:
//! - [`StorageDriver`]: Creates a concrete storage backend from MCU-specific resources.
//! - [`Storage`]: Async API to read, write, and delete typed entries.
//! - [`Entry`]: Defines how a type is tagged and serialized for storage.
//!
//! Entries are identified by an 8-byte base tag (see [`Entry::BASE_TAG`]). Tags can be
//! parameterized via [`Entry::TagParam`] to support multiple stored instances of the same entry
//! type.

mod default;
mod empty;
pub mod flash;
#[cfg(test)]
mod test;

use crate::util::{error, info, warn};
use core::any::Any;
use core::marker::PhantomData;
pub use default::{DefaultReadTransaction, DefaultStorage, DefaultWriteTransaction};
#[cfg(feature = "defmt")]
use defmt::Debug2Format;
use ekv::flash::Flash;
pub use empty::{EmptyReadTransaction, EmptyStorage, EmptyStorageDriver, EmptyWriteTransaction};
use generic_array::{ArrayLength, GenericArray};

/// Error returned by [`ReadTransaction::read`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ValueDeserializeError<E> {
    /// The read value bytes could not be deserialized to the entry type.
    ValueDeserializationFailed,
    /// The stored value has less bytes than expected.
    ValueTooSmall,
    /// Another error occurred.
    Other(E),
}

/// Error returned by [`Cursor::next`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DeserializeError<E> {
    /// The read value bytes could not be deserialized to the entry type.
    ValueDeserializationFailed,
    /// The stored value has less bytes than expected.
    ValueTooSmall,
    /// The read key bytes could not be deserialized to the tag parameter type.
    KeyDeserializationFailed,
    /// The stored key has less bytes than expected.
    KeyTooSmall,
    /// Another error occurred.
    Other(E),
}

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
pub trait Storage {
    /// The flash type of this storage.
    type Flash: Flash;

    /// The read transaction type returned by [`Self::read_transaction`].
    type ReadTransaction<'a>: ReadTransaction<'a, Flash = Self::Flash>
    where
        Self: 'a;

    /// The write transaction type returned by [`Self::write_transaction`].
    type WriteTransaction<'a>: WriteTransaction<'a, Flash = Self::Flash>
    where
        Self: 'a;

    /// Formats the database storage.
    ///
    /// This will format the underlying storage into an empty key-value database. If the storage was
    /// already formatted, all data will be lost.
    fn format(
        &self,
    ) -> impl Future<Output = Result<(), ekv::FormatError<<Self::Flash as Flash>::Error>>>;

    /// Forces eagerly mounting the database storage.
    ///
    /// You don’t have to call this method, mounting is done lazily on first operation.
    ///
    /// Eagerly mounting can still be useful, to detect whether the storage is formatted or not, so
    /// that you can format it if it isn’t before first use.
    fn mount(
        &self,
    ) -> impl Future<Output = Result<(), ekv::MountError<<Self::Flash as Flash>::Error>>>;

    /// Opens a read transaction.
    ///
    /// This will wait if there’s a write transaction either being currently committed, or waiting
    /// to be committed because there are other read transactions open.
    ///
    /// Dropping the ReadTransaction closes the transaction. Make sure to drop it as soon as
    /// possible, to not delay write transaction commits.
    fn read_transaction(&self) -> impl Future<Output = Self::ReadTransaction<'_>>;

    /// Opens a write transaction.
    ///
    /// This will wait if there’s another write transaction already open.
    ///
    /// To make all the writes permanent, call commit. Dropping the WriteTransaction without
    /// committing closes the transaction and discards all written data.
    fn write_transaction(&self) -> impl Future<Output = Self::WriteTransaction<'_>>;
}

/// Error returned by [`mount_or_format`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MountOrFormatError<E> {
    /// Failed to mount storage.
    Mount(ekv::MountError<E>),
    /// Failed to format storage.
    Format(ekv::FormatError<E>),
}

/// Formats the storage database if it is not formatted yet or if it is corrupted, and mounts the storage.
pub async fn mount_or_format<S: Storage>(
    storage: &S,
) -> Result<(), MountOrFormatError<<S::Flash as Flash>::Error>> {
    match storage.mount().await {
        Ok(()) => {
            info!("Successfully mounted storage");
            return Ok(());
        }
        Err(ekv::MountError::Corrupted) => {
            warn!("Storage database is corrupted or not formatted yet. Formatting database...")
        }
        Err(e @ ekv::MountError::Flash(_)) => {
            let format_e = &e;
            #[cfg(feature = "defmt")]
            let format_e = defmt::Debug2Format(&format_e);
            error!("Failed to mount storage: {}", format_e);
            return Err(MountOrFormatError::Mount(e));
        }
    };
    storage
        .format()
        .await
        .map_err(MountOrFormatError::Format)
        .inspect_err(|e| {
            #[cfg(feature = "defmt")]
            let e = Debug2Format(&e);
            error!("Failed to format storage: {}", e);
        })?;
    info!("Successfully formatted storage database");
    storage
        .mount()
        .await
        .map_err(MountOrFormatError::Mount)
        .inspect_err(|e| {
            #[cfg(feature = "defmt")]
            let e = Debug2Format(&e);
            error!("Failed to mount storage: {}", e);
        })?;
    info!("Successfully mounted storage");
    Ok(())
}

/// In-progress read transaction.
pub trait ReadTransaction<'a> {
    /// The flash type of this read transaction.
    type Flash: Flash + 'a;

    /// Reads a single entry from the database.
    fn read<E: Entry>(
        &self,
        tag_param: E::TagParam,
    ) -> impl Future<
        Output = Result<E, ValueDeserializeError<ekv::ReadError<<Self::Flash as Flash>::Error>>>,
    >;

    /// Reads all entries of the given type from the database.
    #[allow(clippy::type_complexity)]
    fn read_all<'b, E: Entry>(
        &'b self,
    ) -> impl Future<
        Output = Result<Cursor<'b, E, Self::Flash>, ekv::ReadError<<Self::Flash as Flash>::Error>>,
    >
    where
        'a: 'b;
}

/// In-progress write transaction.
pub trait WriteTransaction<'a> {
    /// The flash type of this write transaction.
    type Flash: Flash + 'a;

    /// Writes an entry to the database.
    ///
    /// If an entry with the same tag was already present, the previous value is overwritten.
    fn write<E: Entry>(
        &mut self,
        tag_param: E::TagParam,
        entry: &E,
    ) -> impl Future<Output = Result<(), ekv::WriteError<<Self::Flash as Flash>::Error>>>;

    /// Deletes an entry from the database.
    ///
    /// If no entry with the same tag was present, this is a no-op.
    fn delete<E: Entry>(
        &mut self,
        tag_param: E::TagParam,
    ) -> impl Future<Output = Result<(), ekv::WriteError<<Self::Flash as Flash>::Error>>>;

    /// Commit the transaction.
    ///
    /// This will wait until no read transaction is open. While waiting, opening new read
    /// transactions is blocked, to prevent readers from starving writers.
    ///
    /// This makes all the writes permanent on the underlyling database. When it returns, the writes
    /// are guaranteed to be fully safely written to flash, so they can’t get lost by a crash or a
    /// power failure anymore.
    ///
    /// Committing is atomic: if commit is interrupted (due to power loss, crash, or canceling the
    /// future), it is guaranteed that either all or none of the writes in the transaction are
    /// committed.
    fn commit(
        self,
    ) -> impl Future<Output = Result<(), ekv::CommitError<<Self::Flash as Flash>::Error>>>;
}

/// Cursor for a range read.
///
/// This is returned by the [`ReadTransaction::read_all`] method.
pub struct Cursor<'a, E, F: Flash + 'a> {
    inner: ekv::Cursor<'a, F, embassy_sync_0_6::blocking_mutex::raw::CriticalSectionRawMutex>,
    phantom: PhantomData<E>,
}

impl<'a, E: Entry, F: Flash + 'a> Cursor<'a, E, F> {
    /// Gets the next entry, including it's tag parameter.
    ///
    /// If the cursor has reached the end of the iteration, `Ok(None)` is returned.
    pub async fn next(
        &mut self,
    ) -> Result<Option<(E::TagParam, E)>, DeserializeError<ekv::CursorError<F::Error>>> {
        let mut key_buf = GenericArray::<u8, TagSize<E>>::default();
        let mut value_buf = GenericArray::<u8, E::Size>::default();
        match self
            .inner
            .next(&mut key_buf, &mut value_buf)
            .await
            .map_err(DeserializeError::Other)?
        {
            Some((key_len, value_len)) => {
                // Keys are sorted lexicographically and all keys of a given entry type share the
                // same `BASE_TAG` prefix. Once a key with a different prefix is reached, there are
                // no more entries of this type left, so iteration can stop.
                if key_buf.get(..BASE_TAG_SIZE) != Some(&E::BASE_TAG) {
                    return Ok(None);
                }

                // Deserialize entry
                if value_len != <E::Size as typenum::Unsigned>::USIZE {
                    // The value length cannot be greater than E::Size at this point, because the
                    // array that the value is read into is limited to a length of E::Size. In that
                    // case an ekv::ReadError::ValueBufferTooSmall error will already have been
                    // reported.
                    assert!(value_len < <E::Size as typenum::Unsigned>::USIZE);
                    return Err(DeserializeError::ValueTooSmall);
                }
                let entry = E::from_bytes(&value_buf)
                    .ok_or(DeserializeError::ValueDeserializationFailed)?;

                // Deserialize tag parameter
                if key_len != <TagSize<E> as typenum::Unsigned>::USIZE {
                    // The key length cannot be greater than the buffer at this point, because the
                    // buffer is limited to the full tag size. In that case an
                    // ekv::CursorError::KeyBufferTooSmall error will already have been reported.
                    assert!(key_len < <TagSize<E> as typenum::Unsigned>::USIZE);
                    return Err(DeserializeError::KeyTooSmall);
                }
                let tag_param_array = GenericArray::try_from_slice(&key_buf[BASE_TAG_SIZE..])
                    .expect("invalid length of tag parameter bytes");
                let tag_param = E::TagParam::from_bytes(tag_param_array)
                    .ok_or(DeserializeError::KeyDeserializationFailed)?;

                Ok(Some((tag_param, entry)))
            }
            None => Ok(None),
        }
    }
}

/// The number of bytes of the base entry tag.
pub const BASE_TAG_SIZE: usize = 4;

/// The number of bytes of the base entry tag.
pub type BaseTagSize = typenum::U4;

/// The number of bytes of the complete entry tag for the given entry type.
pub type TagSize<E> = typenum::Sum<<<E as Entry>::TagParam as TagParam>::Size, BaseTagSize>;

/// Computes the smallest tag that is greater than every key prefixed with `base_tag`.
///
/// `base_tag` is treated as a big-endian integer and incremented by one. Overflowing the last
/// byte carries into the previous one, so `[1, 2, 255]` becomes `[1, 3, 0]`. If every byte is
/// `0xff` there is no greater `BASE_TAG_SIZE`-byte prefix, so `[0; BASE_TAG_SIZE]` is returned;
/// that case can't be expressed as an exclusive range bound and is rejected at runtime by
/// [`ReadTransaction::read_all`].
pub(crate) const fn base_tag_end(mut base_tag: [u8; BASE_TAG_SIZE]) -> [u8; BASE_TAG_SIZE] {
    let mut i = BASE_TAG_SIZE;
    while i > 0 {
        i -= 1;
        if base_tag[i] != u8::MAX {
            base_tag[i] += 1;
            return base_tag;
        }
        base_tag[i] = 0;
    }
    base_tag
}

/// Builds the storage key for an entry of type `E` with the given tag parameter.
///
/// The key consists of the entry type's [`BASE_TAG`](Entry::BASE_TAG) followed by the serialized
/// `tag_param`.
pub fn entry_tag<E: Entry>(tag_param: E::TagParam) -> GenericArray<u8, TagSize<E>> {
    let mut tag = GenericArray::<u8, TagSize<E>>::default();
    tag[..BASE_TAG_SIZE].copy_from_slice(&E::BASE_TAG);
    tag[BASE_TAG_SIZE..].copy_from_slice(&tag_param.to_bytes());
    tag
}

/// Trait for types that can be stored in persistent storage.
pub trait Entry
where
    TagSize<Self>: ArrayLength,
{
    /// The length of the byte array that this type is serialized to.
    type Size: ArrayLength;

    /// The type used to parameterize entries of this type.
    ///
    /// By settings this to a type that can have different values, multiple instances of the entry
    /// can be stored. If you only ever need to store one instance of this entry type, the type can
    /// be set to `()`.
    type TagParam: TagParam;

    /// The base tag of this entry type, used as a prefix for all of its keys.
    ///
    /// This must be unique across all entry types.
    const BASE_TAG: [u8; BASE_TAG_SIZE];

    /// Deserializes the entry from the specified bytes.
    ///
    /// Returns [`None`] if the entry can not be deserialized.
    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self>
    where
        Self: Sized;

    /// Serializes this entry to a byte array.
    fn to_bytes(&self) -> GenericArray<u8, Self::Size>;
}

/// Parameter to create an entry tag.
pub trait TagParam {
    /// The serialized size of the tag parameter in bytes.
    type Size: ArrayLength + core::ops::Add<BaseTagSize, Output: ArrayLength>;

    /// Serializes the tag parameter to bytes.
    fn to_bytes(&self) -> GenericArray<u8, Self::Size>;

    /// Deserializes the tag parameter from bytes.
    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self>
    where
        Self: Sized;
}

impl TagParam for () {
    type Size = typenum::U0;

    fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
        [].into()
    }

    fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self> {
        let [] = bytes.into_array::<0>();
        Some(())
    }
}

macro_rules! impl_tag_param_for_uint {
    ($ty:ty, $size:path, $size_num:literal) => {
        impl TagParam for $ty {
            type Size = $size;

            fn to_bytes(&self) -> GenericArray<u8, Self::Size> {
                self.to_be_bytes().into()
            }

            fn from_bytes(bytes: &GenericArray<u8, Self::Size>) -> Option<Self> {
                Some(<$ty>::from_be_bytes(bytes.into_array::<$size_num>()))
            }
        }
    };
}

impl_tag_param_for_uint!(u8, typenum::U1, 1);
impl_tag_param_for_uint!(u16, typenum::U2, 2);
impl_tag_param_for_uint!(u32, typenum::U4, 4);
impl_tag_param_for_uint!(u64, typenum::U8, 8);
impl_tag_param_for_uint!(u128, typenum::U16, 16);
