use crate::util::panic;
use core::marker::PhantomData;
use core::ops::Range;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embedded_storage_async::nor_flash::MultiwriteNorFlash;
use generic_array::{ArrayLength, GenericArray};
use sequential_storage::cache::NoCache;
use sequential_storage::map::{MapConfig, MapStorage};
use typenum::Unsigned;

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

#[repr(C)]
struct Buffer<E: Entry, WordSize: ArrayLength> {
    buf1: GenericArray<u8, E::Size>,
    buf2: [u8; ENTRY_TAG_SIZE],
    buf3: GenericArray<u8, WordSize>,
}

impl<E: Entry, WordSize: ArrayLength> Buffer<E, WordSize> {
    fn new() -> Self {
        Self {
            buf1: GenericArray::default(),
            buf2: [0; _],
            buf3: GenericArray::default(),
        }
    }

    unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        let ptr = self as *mut Self as *mut u8;
        unsafe {
            core::slice::from_raw_parts_mut(ptr, E::Size::USIZE + ENTRY_TAG_SIZE + WordSize::USIZE)
        }
    }
}

pub struct Storage<Flash: MultiwriteNorFlash, WordSize: ArrayLength, EraseSize: ArrayLength> {
    inner: Mutex<CriticalSectionRawMutex, MapStorage<[u8; ENTRY_TAG_SIZE], Flash, NoCache>>,
    phantom: PhantomData<(WordSize, EraseSize)>,
}

impl<Flash: MultiwriteNorFlash, WordSize: ArrayLength, EraseSize: ArrayLength>
    Storage<Flash, WordSize, EraseSize>
{
    pub fn new(flash: Flash, flash_range: Range<u32>) -> Self {
        Self {
            inner: Mutex::new(MapStorage::new(
                flash,
                MapConfig::new(flash_range),
                NoCache::new(),
            )),
            phantom: PhantomData,
        }
    }

    pub async fn remove<E: Entry>(
        &self,
        tag_params: E::TagParams,
    ) -> Result<(), Error<Flash::Error>> {
        let mut buf = GenericArray::<u8, EraseSize>::default();

        self.inner
            .lock()
            .await
            .remove_item(&mut buf, &E::tag(tag_params))
            .await
            .map_err(Error::from_sequential_storage)
    }

    pub async fn store<E: Entry>(
        &self,
        tag_params: E::TagParams,
        entry: &E,
    ) -> Result<(), Error<Flash::Error>> {
        let mut buf = Buffer::<E, WordSize>::new();
        let buf = unsafe { buf.as_mut_slice() };

        let value_bytes = entry.to_bytes();

        self.inner
            .lock()
            .await
            .store_item(buf, &E::tag(tag_params), &value_bytes.as_ref())
            .await
            .map_err(Error::from_sequential_storage)
    }

    pub async fn fetch<E: Entry>(
        &self,
        tag_params: E::TagParams,
    ) -> Result<Option<E>, Error<Flash::Error>> {
        let mut buf = Buffer::<E, WordSize>::new();
        let buf = unsafe { buf.as_mut_slice() };

        let data: Option<&[u8]> = self
            .inner
            .lock()
            .await
            .fetch_item(buf, &E::tag(tag_params))
            .await
            .map_err(Error::from_sequential_storage)?;

        Ok(data.and_then(|data| {
            let data = GenericArray::try_from_slice(data).unwrap();
            E::from_bytes(data)
        }))
    }
}
