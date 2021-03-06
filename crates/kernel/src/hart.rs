//! Hart-Local storage

use crate::pmem;
use core::slice;

/// Initialize the hart local storage.
///
/// # Safety
///
/// This function must be called on every hart, after the physical
/// memory allocator is initialized.
pub unsafe fn init_hls() -> Result<(), pmem::alloc::Error> {
    let (start, end) = riscv::symbols::tdata_range();
    let size = end as usize - start as usize;
    let page_count = pmem::alloc::align_up(size, pmem::alloc::PAGE_SIZE) / pmem::alloc::PAGE_SIZE;

    // allocate the new thread local storage and copy the original data
    let orig = slice::from_raw_parts(start, size);
    let mut new = pmem::zalloc_pages(page_count)?;
    new.as_mut()[..size].copy_from_slice(orig);

    // set the thread pointer
    let tp = new.as_ref().as_ptr() as usize;
    asm!("mv tp, {}", in(reg) tp);

    Ok(())
}
