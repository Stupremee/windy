//! Implementation of a Buddy Allocator that is responsible for allocating
//! memory that will then be used by either the slab allocator
//! to allocate objects, or directly by the kernel.

use super::LinkedList;
use crate::utils;
use core::{
    alloc::{AllocError, Layout},
    cmp, fmt, mem,
    ptr::NonNull,
};

/// The number of possible orders.
///
/// Calculated using the following formula:
///     max_size = 2^order_count * order_0_size
pub const ORDER_COUNT: usize = 15;

/// The central structure that is responsible for allocating memory
/// using the buddy algorithm.
pub struct BuddyAllocator {
    orders: [LinkedList; ORDER_COUNT],
}

impl BuddyAllocator {
    /// Create a new, empty buddy allocator.
    pub fn new() -> Self {
        Self {
            orders: [LinkedList::new(); ORDER_COUNT],
        }
    }

    /// Adds a region of memory to this allocator and makes it available
    /// for allocation.
    pub unsafe fn add_heap(&mut self, start: NonNull<u8>, end: NonNull<u8>) {
        // align the pointers
        let start = utils::align_non_null::<_, usize>(start);
        let end = utils::align_non_null::<_, usize>(end);
        assert!(start <= end, "heap start must be before the heap end");

        let end_raw = end.as_ptr() as usize;
        let mut current_start = start.as_ptr() as usize;

        // loop until there's not enough place to fit _at least_ one pointer
        // width into the block.
        while current_start + mem::size_of::<usize>() <= end_raw {
            // get the highest bit of the start that is not zero.
            // 0b0111100011010000 => 0b0100000000000000
            let first_bit = current_start & (!current_start + 1);

            // either we chose the `first_bit` as the size, or, if there's not enough memory left
            // to fit `first_bit` into it, we choose the rest of the memory.
            let size = cmp::min(first_bit, utils::prev_power_of_two(end_raw - current_start));

            // get the order for the new block, and insert the block into our list
            // of free blocks.
            let order = size.trailing_zeros() as usize;

            // limit the biggest possible order to be `ORDER_COUNT - 1`
            let order = cmp::min(order, ORDER_COUNT - 1);

            // we may have decreased the order above, so we also
            // have to update the size if needed
            let size = cmp::min(size, 1 << order);

            self.orders[order].push(current_start as *mut _);

            // go to the next part of the memory.
            log::debug!(
                "size {:x} current_start {:x} end {:x}",
                size,
                current_start,
                end_raw
            );
            current_start += size;
        }
    }

    /// Allocates a chunk of memory that is able to hold the given layout, but may
    /// be bigger than the actual layout size.
    pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        // get the size and order of the layout
        let (_, order) = Self::size_and_order(layout);

        // loop through the orders to find
        // one which can be split up into two buddies
        for idx in order..self.orders.len() {
            let list = &mut self.orders[idx];
            // if there are no blocks for this order, continue
            if list.is_empty() {
                continue;
            }

            // If the found order is larger than the requested order,
            // split the current `order` into two buddies.
            for order_to_split in (order + 1..idx + 1).rev() {
                // we know that the first time we enter this loop, we _must_ have an available
                // block, because otherwise we would have continued already.
                //
                // So pop the block that will be splitted into the buddies
                let block = self.orders[order_to_split].pop().ok_or(AllocError)?;

                // `target_order` is the order where the two buddies from `block` will be put in.
                let target_order = order_to_split - 1;
                let target = &mut self.orders[target_order];

                unsafe {
                    // calculate the address of the buddy for `block`, which will be
                    // the address of the block plus the target order size.
                    let buddy = block as usize + (1 << target_order);

                    // insert the body into the target order
                    target.push(buddy as *mut _);

                    // insert the original block into target order
                    target.push(block);
                }
            }

            // if we get here, we executed all required splits to have
            // at least one block of memory available.
            // So pop it and return it.
            let ptr = self.orders[order]
                .pop()
                .expect("at this point there must be a free block");
            let ptr = NonNull::new(ptr as *mut _).expect("heap block should never be zero");
            return Ok(ptr);
        }

        Err(AllocError)
    }

    /// Deallocates a block of memory, that is located at the given pointer
    /// and has the given layout.
    ///
    /// # Safety
    ///
    /// The pointer (`ptr`) must be allocated by `self` using [`alloc`](Self::alloc).
    pub unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        // get the size and order for the layout
        let (_, order) = Self::size_and_order(layout);

        // put the pointer back into our list of free blocks, in the corresponding order.
        self.orders[order].push(ptr.as_ptr() as *mut usize);

        // now try to merge two buddies.
        let mut current_ptr = ptr.as_ptr() as usize;
        let mut current_order = order;

        // loop through every order, going from low to high order
        while current_order < self.orders.len() {
            // thats a trick to calculate the address for the buddy of the
            // current block.
            let buddy = current_ptr ^ (1 << current_order);

            // loop through every block in the current order,
            // remove the block if it's the buddy for our current block,
            // and set the `buddy_found` flag to `true`.
            let mut buddy_found = false;
            for block in self.orders[current_order].iter_mut() {
                if block.addr() as usize == buddy {
                    block.pop();
                    buddy_found = true;
                    break;
                }
            }

            // if a buddy was found, remove the block, the one we will merge now,
            // from the order, then insert the buddy or current block
            // into the higher order.
            if buddy_found {
                self.orders[current_order].pop();
                current_ptr = cmp::min(current_ptr, buddy);
                current_order += 1;
                self.orders[current_order].push(current_ptr as *mut _);
            } else {
                break;
            }
        }
    }

    /// Returns the `(size, order)` pair for the given layout.
    fn size_and_order(layout: Layout) -> (usize, usize) {
        // `PAGE_SIZE` is the smallest size the allocator can allocate.
        let size = cmp::max(layout.size().next_power_of_two(), super::PAGE_SIZE);
        let order = size.trailing_zeros() as usize;
        (size, order)
    }
}

impl fmt::Debug for BuddyAllocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "BUDDY ALLOCATOR")?;
        writeln!(f, "~~~~~~~~~~~~~~~")?;
        for order in (0..self.orders.len()).rev() {
            let list = &self.orders[order];
            let len = list.iter().count();
            writeln!(f, "Order {} has {} free blocks", order, len)?;
        }
        writeln!(f, "~~~~~~~~~~~~~~~")?;
        Ok(())
    }
}
