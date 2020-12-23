//! Custom Implementation of a slab allocator
//! that is used as the main allocator for the kernel.

use super::LinkedList;
use core::{alloc::AllocError, ptr::NonNull};

/// A slab that holds objects/blocks with a specific size
/// and can allocate and deallocate those objects really fast.
pub struct Slab {
    /// The list of blocks.
    free_list: LinkedList,
    /// Inidicates if this Slab is initialized.
    init: bool,
}

impl Slab {
    /// Creates a new uninitialized `Slab`.
    pub const fn new() -> Self {
        Self {
            free_list: LinkedList::new(),
            init: false,
        }
    }

    /// Initializes this `Slab`, so it becomes ready to allocate and deallocate.
    ///
    /// # Arguments
    /// - `start` the pointer to the memory where this Slab should start
    /// - `block_size` the size for each object of this slab
    /// - `slab_size` the size of memory that is available for this slab.
    pub unsafe fn init(&mut self, start: NonNull<u8>, block_size: usize, slab_size: usize) {
        assert!(
            slab_size >= block_size,
            "Slab must be able to hold at least one block"
        );

        let num_blocks = slab_size / block_size;
        for idx in 0..num_blocks {
            let block = start.as_ptr().add(idx * block_size).cast();
            self.free_list.push(block);
        }

        self.init = true;
    }

    /// Allocates a new block of `block_size` bytes and returns
    /// a pointer to it.
    pub fn alloc(&mut self) -> Result<NonNull<u8>, AllocError> {
        assert!(self.init, "Slab is not initialized.");
        match self.free_list.pop() {
            Some(block) => Ok(block.cast()),
            None => Err(AllocError),
        }
    }

    /// Deallocates a block of memory.
    ///
    /// # Safety
    ///
    /// The `block` has to be allocated by this slab.
    pub unsafe fn dealloc(&mut self, block: NonNull<u8>) {
        assert!(self.init, "Slab is not initialized.");
        self.free_list.push(block.as_ptr() as *mut _);
    }
}
