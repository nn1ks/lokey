use crate::storage::{ENTRY_TAG_SIZE, Entry, Error, Storage};
use core::marker::PhantomData;
use core::ops::Range;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embedded_storage_async::nor_flash::MultiwriteNorFlash;
use generic_array::{ArrayLength, GenericArray};
use sequential_storage::cache::NoCache;
use sequential_storage::map::{MapConfig, MapStorage};
use typenum::Unsigned;

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

pub struct DefaultStorage<Flash, WordSize, EraseSize>
where
    Flash: MultiwriteNorFlash + 'static,
    WordSize: ArrayLength,
    EraseSize: ArrayLength,
{
    inner: Mutex<CriticalSectionRawMutex, MapStorage<[u8; ENTRY_TAG_SIZE], Flash, NoCache>>,
    phantom: PhantomData<(WordSize, EraseSize)>,
}

impl<Flash, WordSize, EraseSize> DefaultStorage<Flash, WordSize, EraseSize>
where
    Flash: MultiwriteNorFlash + 'static,
    WordSize: ArrayLength,
    EraseSize: ArrayLength,
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
}

impl<Flash, WordSize, EraseSize> Storage for DefaultStorage<Flash, WordSize, EraseSize>
where
    Flash: MultiwriteNorFlash + 'static,
    WordSize: ArrayLength,
    EraseSize: ArrayLength,
{
    type FlashError = Flash::Error;

    async fn remove<E: Entry>(&self, tag_params: E::TagParams) -> Result<(), Error<Flash::Error>> {
        let mut buf = GenericArray::<u8, EraseSize>::default();

        self.inner
            .lock()
            .await
            .remove_item(&mut buf, &E::tag(tag_params))
            .await
            .map_err(Error::from_sequential_storage)
    }

    async fn store<E: Entry>(
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

    async fn fetch<E: Entry>(
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
