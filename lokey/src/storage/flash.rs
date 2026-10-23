//! Utility flash implementations.

use core::convert::Infallible;
use core::ops::Range;
use ekv::flash::{Flash, PageID};
use embedded_storage_async::nor_flash;

/// Dummy flash used for [`EmptyStorage`](crate::storage::EmptyStorage).
pub struct EmptyFlash;

impl Flash for EmptyFlash {
    type Error = Infallible;

    fn page_count(&self) -> usize {
        0
    }

    async fn erase(&mut self, _: ekv::flash::PageID) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn read(
        &mut self,
        _: ekv::flash::PageID,
        _: usize,
        _: &mut [u8],
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn write(
        &mut self,
        _: ekv::flash::PageID,
        _: usize,
        _: &[u8],
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Workaround for alignment requirements.
#[repr(C, align(4))]
struct AlignedBuf<const N: usize>([u8; N]);

/// Wraps a [`embedded_storage_async::nor_flash::MultiwriteNorFlash`] and implements
/// [`ekv::flash::Flash`].
pub struct MultiwriteNorFlash<F> {
    flash: F,
    page_range: Range<usize>,
}

impl<F: nor_flash::MultiwriteNorFlash> MultiwriteNorFlash<F> {
    /// Creates a new [`MultiwriteNorFlash`].
    ///
    /// The `page_range` argument specifies what pages in the flash are used for the storage
    /// database.
    pub fn new(flash: F, page_range: Range<usize>) -> Self {
        Self { flash, page_range }
    }

    /// Returns the address of the given access within the flash.
    fn address(&self, page_id: PageID, offset: usize) -> u32 {
        ((self.page_range.start + page_id.index()) * ekv::config::PAGE_SIZE + offset) as u32
    }

    /// Checks whether the given page is within the window.
    fn check_page_range(&self, page_id: PageID) -> Result<(), FlashWindowError<F::Error>> {
        let num_pages = self.page_range.end.saturating_sub(self.page_range.start);
        if page_id.index() >= num_pages {
            return Err(FlashWindowError::PageOutOfRange);
        }
        Ok(())
    }
}

impl<F: nor_flash::MultiwriteNorFlash> Flash for MultiwriteNorFlash<F> {
    type Error = FlashWindowError<F::Error>;

    fn page_count(&self) -> usize {
        self.page_range.end.saturating_sub(self.page_range.start)
    }

    async fn erase(&mut self, page_id: PageID) -> Result<(), Self::Error> {
        self.check_page_range(page_id)?;
        let from = self.address(page_id, 0);
        self.flash
            .erase(from, from + ekv::config::PAGE_SIZE as u32)
            .await
            .map_err(FlashWindowError::Other)
    }

    async fn read(
        &mut self,
        page_id: PageID,
        offset: usize,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.check_page_range(page_id)?;
        let mut buf = AlignedBuf::<{ ekv::config::PAGE_SIZE }>([0; ekv::config::PAGE_SIZE]);
        self.flash
            .read(self.address(page_id, offset), &mut buf.0[..data.len()])
            .await
            .map_err(FlashWindowError::Other)?;
        data.copy_from_slice(&buf.0[..data.len()]);
        Ok(())
    }

    async fn write(
        &mut self,
        page_id: PageID,
        offset: usize,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        self.check_page_range(page_id)?;
        let mut buf = AlignedBuf::<{ ekv::config::PAGE_SIZE }>([0; ekv::config::PAGE_SIZE]);
        buf.0[..data.len()].copy_from_slice(data);
        self.flash
            .write(self.address(page_id, offset), &buf.0[..data.len()])
            .await
            .map_err(FlashWindowError::Other)
    }
}

/// Error returned by methods of [`MultiwriteNorFlash`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FlashWindowError<E> {
    /// The accessed page is outside of the window.
    PageOutOfRange,
    /// Another error occurred.
    Other(E),
}
