use crate::storage::{Entry, Error, Storage, StorageDriver};
use core::convert::Infallible;
use core::marker::PhantomData;

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

pub struct EmptyStorage;

impl Storage for EmptyStorage {
    type FlashError = Infallible;

    async fn remove<E: Entry>(
        &self,
        _tag_params: E::TagParams,
    ) -> Result<(), Error<<Self as Storage>::FlashError>> {
        Ok(())
    }

    async fn store<E: Entry>(
        &self,
        _tag_params: E::TagParams,
        _entry: &E,
    ) -> Result<(), Error<<Self as Storage>::FlashError>> {
        Ok(())
    }

    async fn fetch<E: Entry>(
        &self,
        _tag_params: E::TagParams,
    ) -> Result<Option<E>, Error<<Self as Storage>::FlashError>> {
        Ok(None)
    }
}
