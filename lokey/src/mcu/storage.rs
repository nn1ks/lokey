use alloc::vec;
use alloc::vec::Vec;
use core::ops::Range;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embedded_storage_async::nor_flash::{MultiwriteNorFlash, NorFlash};
use generic_array::{ArrayLength, GenericArray};
use sequential_storage::cache::NoCache;
use sequential_storage::map::{fetch_item, remove_item, store_item};
use typenum::Unsigned;

const ENTRY_TAG_SIZE: usize = 8;

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

trait NorFlashExt {
    /// The largest of the write and read word size
    const WORD_SIZE: usize;
}

impl<F: NorFlash> NorFlashExt for F {
    const WORD_SIZE: usize = if Self::WRITE_SIZE > Self::READ_SIZE {
        Self::WRITE_SIZE
    } else {
        Self::READ_SIZE
    };
}

const fn round_up_to_word_size<F: NorFlash>(value: usize) -> usize {
    let remainder = value % F::WORD_SIZE;
    value + F::WORD_SIZE - remainder
}

pub struct Storage<F> {
    flash: Mutex<CriticalSectionRawMutex, F>,
    flash_range: Range<u32>,
}

impl<F: MultiwriteNorFlash> Storage<F> {
    pub fn new(flash: F, flash_range: Range<u32>) -> Self {
        Self {
            flash: Mutex::new(flash),
            flash_range,
        }
    }

    fn create_buffer<E: Entry>() -> Vec<u8> {
        let buf_len = round_up_to_word_size::<F>(E::Size::USIZE + ENTRY_TAG_SIZE);
        vec![0; buf_len]
    }

    pub async fn remove<E: Entry>(&self, tag_params: E::TagParams) -> Result<(), Error<F::Error>> {
        let mut buf = Self::create_buffer::<E>();
        remove_item(
            &mut *self.flash.lock().await,
            self.flash_range.clone(),
            &mut NoCache::new(),
            &mut buf,
            &E::tag(tag_params),
        )
        .await
        .map_err(Error::from_sequential_storage)
    }

    pub async fn store<E: Entry>(
        &self,
        tag_params: E::TagParams,
        entry: &E,
    ) -> Result<(), Error<F::Error>> {
        let mut buf = Self::create_buffer::<E>();
        let value_bytes = entry.to_bytes();
        store_item(
            &mut *self.flash.lock().await,
            self.flash_range.clone(),
            &mut NoCache::new(),
            &mut buf,
            &E::tag(tag_params),
            &value_bytes.as_ref(),
        )
        .await
        .map_err(Error::from_sequential_storage)
    }

    pub async fn fetch<E: Entry>(
        &self,
        tag_params: E::TagParams,
    ) -> Result<Option<E>, Error<F::Error>> {
        let mut buf = Self::create_buffer::<E>();
        let data: Option<&[u8]> = fetch_item(
            &mut *self.flash.lock().await,
            self.flash_range.clone(),
            &mut NoCache::new(),
            &mut buf,
            &E::tag(tag_params),
        )
        .await
        .map_err(Error::from_sequential_storage)?;
        Ok(data.and_then(|data| {
            let data = GenericArray::try_from_slice(data).unwrap();
            E::from_bytes(data)
        }))
    }
}
