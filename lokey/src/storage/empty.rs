use crate::storage::flash::EmptyFlash;
use crate::storage::{
    Entry, ReadTransaction, Storage, StorageDriver, ValueDeserializeError, WriteTransaction,
};
use core::marker::PhantomData;
use ekv::flash::Flash;

/// A storage driver to crate an [`EmptyStorage`] instance.
pub struct EmptyStorageDriver<M> {
    phantom: PhantomData<M>,
}

impl<Mcu: 'static> StorageDriver for EmptyStorageDriver<Mcu> {
    type Storage = EmptyStorage;
    type Mcu = Mcu;
    type Config = ();

    fn create_storage(_: &'static Self::Mcu, _: Self::Config) -> Self::Storage {
        EmptyStorage
    }
}

/// A storage implementation that does not actually store anything.
///
/// This can be used for testing or for devices that do not have any non-volatile storage.
pub struct EmptyStorage;

impl Storage for EmptyStorage {
    type Flash = EmptyFlash;

    type ReadTransaction<'a> = EmptyReadTransaction;

    type WriteTransaction<'a> = EmptyWriteTransaction;

    async fn format(&self) -> Result<(), ekv::FormatError<<Self::Flash as Flash>::Error>> {
        Ok(())
    }

    async fn mount(&self) -> Result<(), ekv::MountError<<Self::Flash as Flash>::Error>> {
        Ok(())
    }

    async fn read_transaction(&self) -> Self::ReadTransaction<'_> {
        EmptyReadTransaction
    }

    async fn write_transaction(&self) -> Self::WriteTransaction<'_> {
        EmptyWriteTransaction
    }
}

/// Read transaction of [`EmptyStorage`].
pub struct EmptyReadTransaction;

impl<'a> ReadTransaction<'a> for EmptyReadTransaction {
    type Flash = EmptyFlash;

    async fn read<E: Entry>(
        &self,
        _: E::TagParam,
    ) -> Result<E, ValueDeserializeError<ekv::ReadError<<Self::Flash as Flash>::Error>>> {
        Err(ValueDeserializeError::Other(ekv::ReadError::KeyNotFound))
    }

    async fn read_all<'b, E: Entry>(
        &'b self,
    ) -> Result<super::Cursor<'b, E, Self::Flash>, ekv::ReadError<<Self::Flash as Flash>::Error>>
    where
        'a: 'b,
    {
        Err(ekv::ReadError::KeyNotFound)
    }
}

/// Write transaction of [`EmptyStorage`].
pub struct EmptyWriteTransaction;

impl<'a> WriteTransaction<'a> for EmptyWriteTransaction {
    type Flash = EmptyFlash;

    async fn write<E: Entry>(
        &mut self,
        _: E::TagParam,
        _: &E,
    ) -> Result<(), ekv::WriteError<<Self::Flash as Flash>::Error>> {
        Ok(())
    }

    async fn delete<E: Entry>(
        &mut self,
        _: E::TagParam,
    ) -> Result<(), ekv::WriteError<<Self::Flash as Flash>::Error>> {
        Ok(())
    }

    async fn commit(self) -> Result<(), ekv::CommitError<<Self::Flash as Flash>::Error>> {
        Ok(())
    }
}
