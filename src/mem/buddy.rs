//! Implementation of a Buddy Allocator that is responsible for allocating
//! memory that will then be used by either the slab allocator
//! to allocate objects, or directly by the kernel.

use super::{AllocStats, LinkedList};
use core::{alloc::AllocError, cmp, fmt, mem, ptr::NonNull};

/// The maximum order for the buddy allocator. (inclusive).
///
/// Calculated using the following formula:
///     max_size = 2^order_count * order_0_size
pub const MAX_ORDER: usize = 14;

/// The size of the orders array inside the buddy allocator.
///
/// We add `1` here because this is the size of the array.
pub const ORDER_COUNT: usize = 14 + 1;

/// Calculates the number of bytes inside the `order`.
pub fn size_for_order(order: usize) -> usize {
    (1 << order) * super::PAGE_SIZE
}

/// The central structure that is responsible for allocating memory
/// using the buddy algorithm.
pub struct BuddyAllocator {
    orders: [LinkedList; ORDER_COUNT],
    stats: AllocStats,
}

impl BuddyAllocator {
    /// Create a new, empty buddy allocator.
    pub fn new() -> Self {
        Self {
            orders: [LinkedList::new(); ORDER_COUNT],
            stats: AllocStats::with_name("Buddy Allocator"),
        }
    }

    /// Adds a region of memory to this allocator and makes it available
    /// for allocation.
    pub unsafe fn add_heap(&mut self, start: NonNull<u8>, end: NonNull<u8>) {
        let mut start = start.as_ptr();
        let end = end.as_ptr();

        while (end as usize - start as usize) >= super::PAGE_SIZE {
            let order = self.add_single_heap_region(start, end);
            start = start.add(size_for_order(order));
        }
    }

    /// Adds a single order that fits into the given range into this allocator.
    ///
    /// # Returns
    ///
    /// The order the range was inserted in.
    unsafe fn add_single_heap_region(&mut self, start: *mut u8, end: *mut u8) -> usize {
        // align the pointers
        let start = start.wrapping_add(start.align_offset(mem::align_of::<usize>()));
        let start_addr = start as usize;

        let end = end.wrapping_add(end.align_offset(mem::align_of::<usize>()));
        let end_addr = end as usize;

        // do certain checks that are required
        assert!(start <= end, "heap start must be before the heap end");
        assert!(
            (end_addr - start_addr) >= super::PAGE_SIZE,
            "heap region must be at least one page size"
        );

        // loop through every order, and check if the `start..end` region can fit
        // the order.
        let mut order = 0;
        while order <= MAX_ORDER {
            // calculate the size, that the next order would require,
            // so we can break if not, and we will use the previous order,
            // that fits into the range.
            let order_size = size_for_order(order + 1);

            // add the order size to the start address
            let new_end = match start_addr.checked_add(order_size) {
                Some(num) => num,
                // if we overflow
                None => break,
            };

            // now check if the region is large enough to store
            // the order size
            if new_end <= end_addr {
                // if the new end is smaller than the end address,
                // the heap region is large enough and we try the next order
                order += 1;
            } else {
                // heap region too small, so use the previous order
                break;
            }
        }

        // update the alloc statistics
        self.stats.total += size_for_order(order);

        // push the block to our block list.
        self.orders[order].push(start as *mut _);
        order
    }

    /// Allocates a chunk of memory that has the given order.
    ///
    /// The size for the given chunk is defined by the `order`, calculated using:
    /// ```ignore
    /// size = (1 << order) * PAGE_SIZE
    /// ```
    pub fn alloc(&mut self, order: usize) -> Result<NonNull<u8>, AllocError> {
        if order > MAX_ORDER {
            return Err(AllocError);
        }

        // udate the allocator statistics
        self.stats.requested += size_for_order(order);
        self.stats.allocated += size_for_order(order);

        // fast path: if a block with the requested order exists,
        // return it
        if let Some(block) = self.orders[order].pop() {
            let ptr = NonNull::new(block as *mut u8).expect("block pointer shoulnd't be zero");
            return Ok(ptr);
        }

        // slow path: find a order that we can split into two buddies,
        // split the order, return one of the buddies.
        let split_idx = self
            .orders
            .iter()
            .position(|list| !list.is_empty())
            .ok_or(AllocError)?;

        // now walk down the orders from top to bottom, so we can split
        // multiple orders if necessary
        for order_to_split in (order + 1..split_idx + 1).rev() {
            // we can `unwrap` here, because we _must_ have a block available,
            // otherwise we wouldn't be here.
            //
            // `block` is the block that we will split into two buddies
            let block = self.orders[order_to_split].pop().unwrap();

            // `target` is the order where both of the buddies will end up in
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
                let buddy_addr = (block as usize) + size_for_order(target_order);

                // now insert both bodies into the target order
                target.push(block);
                target.push(buddy_addr as *mut _);
            }
        }

        // now pop one of the created buddies from the list.
        // we must have one, otherwise we either would have returned
        // already, or a buddy would've been created
        let ptr = self.orders[order]
            .pop()
            .expect("there must be a buddy available");
        let ptr = NonNull::new(ptr as *mut _).expect("heap block should never be zero");
        Ok(ptr)
    }

    /// Deallocates a block of memory, that is located at the given pointer
    /// and was allocated in the given order.
    ///
    /// # Safety
    ///
    /// The pointer (`ptr`) must be allocated by `self` using [`alloc`](Self::alloc).
    /// `order` must be a valid order, otherwise a panic will happen.
    pub unsafe fn dealloc(&mut self, ptr: NonNull<u8>, mut order: usize) {
        assert!(order <= MAX_ORDER, "invalid order given to dealloc");

        // first, insert the buddy into it's order
        self.orders[order].push(ptr.as_ptr() as _);

        // the pointer to the block we are currently trying to merge.
        let mut ptr = ptr.as_ptr() as usize;

        // now we try to merge two buddies if they are both free.
        for target_order in order + 1..self.orders.len() {
            // this is a trick to find the address for the other buddy, if we
            // have the address of one of the buddies.
            let buddy_addr = ptr ^ size_for_order(order);

            // now try to find the buddy in the free orders list
            if let Some(buddy) = self.orders[order]
                .iter_mut()
                .find(|buddy| buddy.addr() as usize == buddy_addr)
            {
                // if we found the buddy, remove it
                buddy.pop();
                // and pop the other buddy so we remove both buddies,
                // since they will be merged together
                self.orders[order].pop();
                // after removing both buddies,
                // push the merged block to the next oder
                self.orders[target_order].push(ptr as *mut _);
                // now just update the `ptr` to the new block
                ptr = cmp::min(buddy_addr, ptr);
                // and go to the next order
                order += 1;
            } else {
                // otherwise we break because we are not able
                // to do any additional merges.
                break;
            }
        }
    }

    /// Returns a copy of the stats at the moment for this allocator.
    pub fn stats(&self) -> AllocStats {
        self.stats.clone()
    }
}
