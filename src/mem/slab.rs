//! Custom Implementation of a slab allocator
//! that is used as the main allocator for the kernel.

use core::{alloc::AllocError, ptr::NonNull};

pub struct Slab {
    /// The list of blocks.
    free_list: Option<NonNull<Block>>,
    /// The number of free blocks.
    free_list_len: usize,
}

impl Slab {
    /// Creates a new `Slab` that starts at the given address, has `slab_size` bytes available
    /// and should contain `block_size` big blocks.
    pub unsafe fn new(start: NonNull<u8>, slab_size: usize, block_size: usize) -> Self {
        assert!(
            slab_size >= block_size,
            "slab size has to be greater or equal to block size"
        );

        let num_blocks = slab_size / block_size;

        let mut head = start.cast::<Block>();
        for idx in 1..num_blocks {
            let head = head.as_mut();

            let next = start.as_ptr().add(idx * block_size).cast();
            // TODO: This can be replaced by `new_unchecked`
            head.next = Some(NonNull::new(next).unwrap());
        }

        Self {
            free_list: Some(head),
            free_list_len: num_blocks,
        }
    }

    /// Allocates a new block of `block_size` bytes and returns
    /// a pointer to it.
    pub fn alloc(&mut self) -> Result<NonNull<u8>, AllocError> {
        match self.pop_block() {
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
        self.push_block(block.cast());
    }

    fn pop_block(&mut self) -> Option<NonNull<Block>> {
        self.free_list.take().map(|mut head| {
            self.free_list = unsafe { head.as_mut() }.next;
            self.free_list_len -= 1;
            head
        })
    }

    unsafe fn push_block(&mut self, mut block: NonNull<Block>) {
        self.free_list_len += 1;
        block.as_mut().next = self.free_list.take();
        self.free_list = Some(block);
    }
}

pub struct Block {
    next: Option<NonNull<Block>>,
}
