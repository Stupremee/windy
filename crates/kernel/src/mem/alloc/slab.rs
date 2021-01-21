//! Implementation of a Slab Allocator that
//! is part of the main allocator for the kernel.

use super::{AllocStats, Error, Result};
use crate::mem::LinkedList;
use core::ptr::NonNull;

/// A slab holds many objects/blocks with a fixed size
/// and can alloate and deallocate those objects really fast.
pub struct Slab {
    free_list: LinkedList,
    block_size: usize,
    stats: AllocStats,
}

impl Slab {
    /// Creates a new [`Slab`] that will contain blocks
    /// with the given, fixed size.
    ///
    /// # Safety
    /// `block_size` must be a power of two.
    pub const unsafe fn new(block_size: usize) -> Self {
        Self {
            free_list: LinkedList::new(),
            stats: AllocStats::with_name("Slab Allocator"),
            block_size,
        }
    }

    /// Adds a region of memory to this slab, which will be available
    /// for allocation.
    ///
    /// # Safety
    ///
    /// - `start` and `end` must be aligned to the block size of this slab.
    /// - the region must be valid to write for the entire lifetime of this allocator.
    pub unsafe fn add_region(&mut self, start: NonNull<u8>, end: NonNull<u8>) {
        let mut ptr = start.as_ptr();

        while (ptr.add(self.block_size)) <= end.as_ptr() {
            let block = NonNull::new_unchecked(ptr as *mut _);

            self.free_list.push(block);
            self.stats.total += self.block_size;

            ptr = ptr.add(self.block_size);
        }
    }

    /// Allocates a new block with the fixed size that was specified in
    /// [`Slab::new`]
    pub fn allocate(&mut self) -> Result<NonNull<[u8]>> {
        let block = self.free_list.pop().ok_or(Error::NoMemoryAvailable)?;
        let slice = core::ptr::slice_from_raw_parts_mut(block.as_ptr().cast(), self.block_size);
        Ok(unsafe { NonNull::new_unchecked(slice) })
    }

    /// Deallocates a block of memory.
    ///
    /// # Safety
    ///
    /// The `block` has to be allocated by this slab.
    pub unsafe fn deallocate(&mut self, block: NonNull<u8>) {
        self.free_list.push(block.cast());
    }

    /// Return the size of the blocks that this slab
    /// is able to allocate.
    pub fn block_size(&self) -> usize {
        self.block_size
    }

    /// Returns a copy of the statistics for this allocator.
    pub fn stats(&self) -> AllocStats {
        self.stats.clone()
    }
}
