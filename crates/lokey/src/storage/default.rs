use super::Cursor;
use crate::storage::{
    Entry, ReadTransaction, Storage, ValueDeserializeError, WriteTransaction, base_tag_end,
    entry_tag,
};
use core::marker::PhantomData;
use ekv::Database;
use ekv::flash::Flash;
use generic_array::GenericArray;

/// The default storage.
pub struct DefaultStorage<F>
where
    F: Flash,
{
    database: Database<F, embassy_sync_0_6::blocking_mutex::raw::CriticalSectionRawMutex>,
}

impl<F> DefaultStorage<F>
where
    F: Flash,
{
    /// Creates a new [`DefaultStorage`] instance.
    pub fn new(flash: F, random_seed: u32) -> Self {
        let mut config = ekv::Config::default();
        config.random_seed = random_seed;
        Self {
            database: Database::new(flash, config),
        }
    }
}

impl<F> Storage for DefaultStorage<F>
where
    F: Flash,
{
    type Flash = F;

    type ReadTransaction<'a>
        = DefaultReadTransaction<'a, Self::Flash>
    where
        Self: 'a;

    type WriteTransaction<'a>
        = DefaultWriteTransaction<'a, Self::Flash>
    where
        Self: 'a;

    async fn format(&self) -> Result<(), ekv::FormatError<<Self::Flash as Flash>::Error>> {
        self.database.format().await
    }

    async fn mount(&self) -> Result<(), ekv::MountError<<Self::Flash as Flash>::Error>> {
        self.database.mount().await
    }

    async fn read_transaction(&self) -> Self::ReadTransaction<'_> {
        DefaultReadTransaction {
            inner: self.database.read_transaction().await,
        }
    }

    async fn write_transaction(&self) -> Self::WriteTransaction<'_> {
        DefaultWriteTransaction {
            inner: self.database.write_transaction().await,
        }
    }
}

/// Read transaction of [`DefaultStorage`].
pub struct DefaultReadTransaction<'a, F: Flash + 'a> {
    inner:
        ekv::ReadTransaction<'a, F, embassy_sync_0_6::blocking_mutex::raw::CriticalSectionRawMutex>,
}

impl<'a, F: Flash + 'a> ReadTransaction<'a> for DefaultReadTransaction<'a, F> {
    type Flash = F;

    async fn read<E: Entry>(
        &self,
        tag_param: E::TagParam,
    ) -> Result<E, ValueDeserializeError<ekv::ReadError<<Self::Flash as Flash>::Error>>> {
        let key = entry_tag::<E>(tag_param);
        let mut value_buf = GenericArray::<u8, E::Size>::default();
        let len = self
            .inner
            .read(&key, &mut value_buf)
            .await
            .map_err(ValueDeserializeError::Other)?;
        if len != <E::Size as typenum::Unsigned>::USIZE {
            // The value length cannot be greater than E::Size at this point, because the array that
            // the value is read into is limited to a length of E::Size. In that case an
            // ekv::ReadError::BufferTooSmall error will already have been reported.
            assert!(len < <E::Size as typenum::Unsigned>::USIZE);
            return Err(ValueDeserializeError::ValueTooSmall);
        }
        E::from_bytes(&value_buf).ok_or(ValueDeserializeError::ValueDeserializationFailed)
    }

    async fn read_all<'b, E: Entry>(
        &'b self,
    ) -> Result<Cursor<'b, E, Self::Flash>, ekv::ReadError<<Self::Flash as Flash>::Error>>
    where
        'a: 'b,
    {
        // Every key of this entry type is `BASE_TAG` followed by the serialized tag parameter, so
        // `BASE_TAG..base_tag_end(BASE_TAG)` covers exactly the keys of this entry type. The
        // upper bound also keeps the cursor from reaching the next entry type's keys, whose values
        // may not fit the value buffer of `E`.
        // An all-`0xff` base tag has no greater prefix, so no exclusive upper bound exists.
        assert_ne!(
            E::BASE_TAG,
            base_tag_end(E::BASE_TAG),
            "BASE_TAG must not be all 0xff"
        );

        // Evaluated as a const so that the bounds are promoted to `'static` and don't borrow from
        // this function's locals.
        let bounds = &const { (E::BASE_TAG, base_tag_end(E::BASE_TAG)) };

        Ok(Cursor {
            inner: self
                .inner
                .read_range(bounds.0.as_slice()..bounds.1.as_slice())
                .await?,
            phantom: PhantomData,
        })
    }
}

/// Write transaction of [`DefaultStorage`].
pub struct DefaultWriteTransaction<'a, F: Flash> {
    inner: ekv::WriteTransaction<
        'a,
        F,
        embassy_sync_0_6::blocking_mutex::raw::CriticalSectionRawMutex,
    >,
}

impl<'a, F: Flash + 'a> WriteTransaction<'a> for DefaultWriteTransaction<'a, F> {
    type Flash = F;

    async fn write<E: Entry>(
        &mut self,
        tag_param: E::TagParam,
        entry: &E,
    ) -> Result<(), ekv::WriteError<<Self::Flash as Flash>::Error>> {
        let key = entry_tag::<E>(tag_param);
        let value = entry.to_bytes();
        self.inner.write(&key, &value).await
    }

    async fn delete<E: Entry>(
        &mut self,
        tag_param: E::TagParam,
    ) -> Result<(), ekv::WriteError<<Self::Flash as Flash>::Error>> {
        let key = entry_tag::<E>(tag_param);
        self.inner.delete(&key).await
    }

    async fn commit(self) -> Result<(), ekv::CommitError<<Self::Flash as Flash>::Error>> {
        self.inner.commit().await
    }
}
