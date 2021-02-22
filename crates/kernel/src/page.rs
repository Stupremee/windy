//! Implementation of the paging system.

mod types;
pub use types::{PageSize, Perm, PhysAddr, VirtAddr};

pub mod sv39;

use riscv::csr::satp;

displaydoc_lite::displaydoc! {
    /// Errors that are related to paging.
    #[derive(Debug)]
    pub enum Error {
        /// tried to map an address that is not aligned to the page size
        UnalignedAddress,
        /// tried to identity map a range using a page size that can't fit into the range
        RangeTooSmall,
        /// failed to allocate a new page
        Alloc(crate::pmem::AllocError),
    }
}

/// Return a exclusive reference to the page table that
/// the `satp` register points to.
pub unsafe fn root() -> &'static mut sv39::Table {
    let addr = satp::read().root_table;
    &mut *(addr as usize as *mut _)
}

/// Map the given address using the root page table.
pub unsafe fn map(
    paddr: PhysAddr,
    vaddr: VirtAddr,
    size: PageSize,
    perm: Perm,
) -> Result<(), Error> {
    root().map(paddr, vaddr, size, perm)?;
    riscv::asm::sfence(usize::from(vaddr), None);
    Ok(())
}

/// Identity map the given range using `size` pages.
pub unsafe fn identity_map(
    start: PhysAddr,
    end: PhysAddr,
    perm: Perm,
    size: PageSize,
) -> Result<(), Error> {
    root().identity_map(start, end, perm, size)
}

/// Unmap the given virtual address. Returns `true` if the page was unmapped,
/// `false` if there's no mapped entry at the given virt addr.
pub unsafe fn unmap(vaddr: VirtAddr) -> bool {
    let res = root().unmap(vaddr);
    riscv::asm::sfence(usize::from(vaddr), None);
    res
}
