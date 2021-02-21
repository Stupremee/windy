//! Implementation of the paging system.

mod types;
pub use types::{PageSize, Perm, PhysAddr, VirtAddr};

pub mod sv39;

displaydoc_lite::displaydoc! {
    /// Errors that are related to paging.
    #[derive(Debug)]
    pub enum Error {
        /// Tried to map an address that is not aligned to the page size.
        UnalignedAddress,
        /// Failed to allocate a new page.
        Alloc(crate::pmem::AllocError),
    }
}

/// Return a exclusive reference to the page table that
/// the `satp` register points to.
pub unsafe fn root() -> &'static mut sv39::Table {
    todo!()
}
