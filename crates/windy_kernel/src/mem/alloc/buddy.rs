//! Implementation of a Buddy Allocator that is responsible for allocating
//! the physical memory that will then be used by either the slab allocator
//! to allocate objects, or directly by the kernel.

use super::{AllocStats, Error, LinkedList, Result};
use core::{cmp, mem, ptr, ptr::NonNull};

// FIXME: We totally ignore the alignment currently.

/// The maximum order for the buddy allocator (inclusive).
pub const MAX_ORDER: usize = 14;

/// The size of the orders array inside the buddy allocator.
pub const ORDER_COUNT: usize = MAX_ORDER + 1;

const EMPTY_ORDERS: [LinkedList; ORDER_COUNT] = [LinkedList::new(); ORDER_COUNT];

/// Calculates the size in bytes for the given order.
pub fn size_for_order(order: usize) -> usize {
    (1 << order) * super::PAGE_SIZE
}

/// The central structure that is responsible for allocating
/// memory using the buddy allocation algorithm.
pub struct BuddyAllocator {
    orders: [LinkedList; ORDER_COUNT],
    stats: AllocStats,
}

impl BuddyAllocator {
    /// Create a empty and uninitialized buddy allocator.
    pub fn new() -> Self {
        Self {
            orders: EMPTY_ORDERS,
            stats: AllocStats::with_name("Buddy Allocator"),
        }
    }

    /// Adds a region of memory to this allocator and makes it available for allocation.
    ///
    /// This method will add as many orders as possible, meaning that a region of size
    /// `size_for_order(4) + 4KiB` will add one order `4` page and one order `0` page.
    /// If the region size is not a multiple of the [pagesize](super::PAGE_SIZE),
    /// the memory that is leftover will stay untouuched.
    ///
    /// If the `start` pointer is not aligned to the word size it will be aligned
    /// correctly before added to this allocator.
    ///
    /// Returns the total number of bytes that were added to this allocator.
    ///
    /// # Safety
    ///
    /// `start` and `end` must be valid to write for the entire lifetime of this allocator.
    pub unsafe fn add_region(&mut self, start: NonNull<u8>, end: NonNull<u8>) -> Result<usize> {
        // align the pointer
        let start = start.as_ptr();
        let mut start = start.wrapping_add(start.align_offset(mem::align_of::<usize>()));
        let end = end.as_ptr();

        // check if there's enough memory for at least
        // one page
        if (end as usize - start as usize) < super::PAGE_SIZE {
            return Err(Error::RegionTooSmall);
        }

        // check if the memory region is invalid
        if end < start {
            return Err(Error::InvalidRegion);
        }

        // loop until there's not enough memory left to allocate a single page
        let mut total = 0;
        while (end as usize - start as usize) >= super::PAGE_SIZE {
            let order = self.add_single_region(start, end);
            let size = size_for_order(order);

            start = start.add(size);
            total += size;
        }

        // update statistics
        self.stats.total += total;
        Ok(total)
    }

    /// Tries to add a single order to this allocator from the given range.
    ///
    /// Returns the order which was inserted into this allocator.
    unsafe fn add_single_region(&mut self, start: *mut u8, end: *mut u8) -> usize {
        let start_addr = start as usize;

        // loop until we reached the maximum order
        let mut order = 0;
        while order <= MAX_ORDER {
            // calculate the size for the next order,
            // so we can break if another order doesn't fit.
            let size = size_for_order(order + 1);

            // check if there's enough memory left for the size of
            // the next order
            let new_end = match start_addr.checked_add(size) {
                Some(num) => num,
                None => break,
            };

            // if there is enough place, try the next order,
            // otherwise we break
            if new_end <= end as usize {
                order += 1;
            } else {
                break;
            }
        }

        // push the block to the list for the given order
        let ptr = NonNull::new_unchecked(start as *mut _);
        self.orders[order].push(ptr);
        order
    }

    /// Allocates a chunk of memory that has the given order.
    ///
    /// The size for returned chunk can be calculated using [`size_for_order`].
    pub fn allocate(&mut self, order: usize) -> Result<NonNull<[u8]>> {
        if order > MAX_ORDER {
            return Err(Error::OrderTooLarge);
        }

        // update statistics
        self.stats.requested += size_for_order(order);
        self.stats.allocated += size_for_order(order);

        // fast path: if there's a block with the given order,
        // return it
        if let Some(block) = self.orders[order].pop() {
            let len = size_for_order(order);
            let ptr = unsafe {
                NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(block.as_ptr().cast(), len))
            };
            return Ok(ptr);
        }

        // slow path: find an order we can split into two buddies.
        // find first list that is not empty
        let split_idx = self
            .orders
            .iter()
            .position(|list| !list.is_empty())
            .ok_or(Error::NoMemoryAvailable)?;

        // now walk down the orders from top to bottom,
        // so we can split multiple orders if necessary
        for order_to_split in (order + 1..split_idx + 1).rev() {
            // there _must_ be at least one block, because either this is the first,
            // non-empty list or we have splitted two buddies from the previous order.
            let block = self.orders[order_to_split].pop().unwrap();

            // target is the order where both buddies will end up in
            let target_order = order_to_split - 1;
            let target = &mut self.orders[target_order];

            unsafe {
                // if this is how the order before the split looked like:
                //
                // +-- this is were `block` starts
                // v
                // +--------------------------------+
                // |        `order_to_split`        |
                // +--------------------------------+
                //
                // so to get the buddy address we do the following:
                //
                // +-- this is were `block` starts, it's now the first buddy
                // v
                // +---------------------------------+
                // |    buddy 1     |    buddy 2     |
                // +---------------------------------+
                //                  ^
                //                  +--- `buddy_addr` is here, we calculate it by using the `block`
                //                       address plus the size of the target order
                let buddy_addr = (block.as_ptr() as usize) + size_for_order(target_order);

                // now insert both bodies into the target order
                target.push(block);
                target.push(NonNull::new_unchecked(buddy_addr as *mut _));
            }
        }

        // now pop one of the buddies that was created and cast it to a slice.
        let ptr = self.orders[order]
            .pop()
            .ok_or(Error::NoMemoryAvailable)?
            .as_ptr()
            .cast::<u8>();
        let len = size_for_order(order);

        // SAFETY
        // Inside the `self.orders` list are only `NonNull` pointers
        // because heap regions must always be non null.
        let ptr = unsafe { NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(ptr, len)) };
        Ok(ptr)
    }

    /// Deallocates a block of memory, that was allocated using the given order.
    ///
    /// # Safety
    ///
    /// The poitner must be allocated by `self` using the [`Self::allocate`] method
    /// with the same order as given here.
    pub unsafe fn deallocate(&mut self, ptr: NonNull<u8>, mut order: usize) {
        // insert the raw block into our free list
        self.orders[order].push(ptr.cast());

        // the address of the block we are currently
        // trying to merge.
        let mut ptr = ptr.as_ptr() as usize;

        // try to merge two buddies if both of them are free.
        // loop at the next order after the given one, until the maximum
        // order, so that we are able to merge multiple times if possible
        for target_order in order + 1..self.orders.len() {
            // this is a trick to find the address for the other buddy
            // if you have the address of one of them
            let buddy_addr = ptr ^ size_for_order(order);

            // check if the buddy is free by searching inside the list
            // of the current order
            if let Some(buddy) = self.orders[order].iter_mut().find(|buddy| {
                buddy
                    .as_ptr()
                    .map_or(false, |addr| addr.as_ptr() as usize == buddy_addr)
            }) {
                // if we found a buddy, remove it
                buddy.pop();

                // push the merged buddy into the target order (the next order)
                //
                // SAFETY
                // There can never be one buddy that is at address 0.
                let merged_buddy = NonNull::new_unchecked(ptr as *mut _);
                self.orders[target_order].push(merged_buddy);

                // update `ptr` to point to the new block,
                // which is the smaller address of both buddies
                ptr = cmp::min(buddy_addr, ptr);
                // go to the next order so we can try to merge another buddies
                order += 1;
            } else {
                break;
            }
        }
    }

    /// Return a copy of the statistics for this allocator.
    pub fn stats(&self) -> AllocStats {
        self.stats.clone()
    }
}
