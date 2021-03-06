//! Interaction with physical memory.

mod rangeset;
pub use rangeset::{Error as RangeError, Range, RangeSet};

pub mod linked_list;
pub use linked_list::LinkedList;

pub mod alloc;
pub use self::alloc::Error as AllocError;

use crate::unit;
use core::{array, ptr::NonNull};
use devicetree::DeviceTree;

displaydoc_lite::displaydoc! {
    /// Errors that are related to memory management.
    #[derive(Debug)]
    pub enum Error {
        /// {_0}
        RangeSet(RangeError),
        /// {_0}
        Alloc(AllocError),
        /// tried to add a memory region that starts at `0`.
        NullRegion,
    }
}

/// Initialize the global memory allocator.
///
/// Return the list of
pub unsafe fn init(tree: &DeviceTree<'_>) -> Result<RangeSet, Error> {
    let mut memory = RangeSet::new();

    tree.memory()
        .regions()
        .try_for_each(|region| {
            let range = Range::new(region.start(), region.end() - 1);
            memory.insert(range)?;
            Ok::<(), RangeError>(())
        })
        .map_err(Error::RangeSet)?;

    array::IntoIter::new(get_blocked_ranges(tree))
        .try_for_each(|range| {
            memory.remove_range(range)?;
            Ok(())
        })
        .map_err(Error::RangeSet)?;

    memory
        .as_slice()
        .iter()
        .try_for_each(|&Range { start, end }| {
            debug!(
                "Making region {:#X}..{:#X} available for allocation",
                start, end
            );

            let start = NonNull::new(start as *mut _).ok_or(Error::NullRegion)?;
            let end = NonNull::new(end as *mut _).ok_or(Error::NullRegion)?;
            alloc::allocator()
                .add_region(start, end)
                .map_err(Error::Alloc)?;

            Ok::<(), Error>(())
        })?;

    info!(
        "{} the physical memory allocator with {} free memory",
        "Initialized".green(),
        unit::bytes(alloc::allocator().stats().total),
    );

    Ok(memory)
}

/// Get a list of memory ranges that must not be used for memory allocation,
/// like the kernel itself and OpenSBI.
fn get_blocked_ranges(tree: &DeviceTree<'_>) -> [Range; 3] {
    let (kernel_start, kernel_end) = riscv::symbols::kernel_range();

    // we align the end of the device tree to 4KiB to map them later
    let fdt = tree.as_ptr() as usize;
    let fdt_end = alloc::align_up(fdt + tree.total_size() as usize, alloc::PAGE_SIZE) - 1;

    [
        // this range contains the OpenSBI firmware
        Range::new(0x8000_0000, 0x801F_FFFF),
        // the kernel itself
        Range::new(kernel_start as _, kernel_end as usize - 1),
        // the actual device tree
        Range::new(fdt, fdt_end),
    ]
}

/// Return the statistics for the physical memory allocator.
pub fn alloc_stats() -> alloc::AllocStats {
    alloc::allocator().stats()
}

/// Allocate a single page of physical memory.
pub fn alloc() -> Result<NonNull<[u8]>, AllocError> {
    alloc::allocator().alloc()
}

/// Deallocate the given page.
pub unsafe fn dealloc(ptr: NonNull<u8>) {
    alloc::allocator().dealloc(ptr)
}

/// Deallocate `count` number of pages that were allocated by [`alloc_pages`].
pub unsafe fn dealloc_pages(ptr: NonNull<u8>, pages: usize) {
    alloc::allocator().dealloc_pages(ptr, pages)
}

/// Allocate a multiple pages of physical memory, that are contigous.
pub fn alloc_pages(count: usize) -> Result<NonNull<[u8]>, AllocError> {
    alloc::allocator().alloc_pages(count)
}

/// Allocate a single page of physical memory, and initialize all bytes with zero.
pub fn zalloc() -> Result<NonNull<[u8]>, AllocError> {
    zalloc_pages(1)
}

/// Allocate a multiple pages of physical memory, that are contigous,
/// and initialize all bytes with zero.
pub fn zalloc_pages(count: usize) -> Result<NonNull<[u8]>, AllocError> {
    let ptr = alloc::allocator().alloc_pages(count)?;

    let slice = ptr.cast::<u64>().as_ptr();
    for off in 0..((count * alloc::PAGE_SIZE) / 8) {
        unsafe {
            *slice.add(off) = 0;
        }
    }

    Ok(ptr)
}
