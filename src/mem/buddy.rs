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
        let start_addr = start.as_ptr() as usize;

        let end = utils::align_non_null::<_, usize>(end);
        let end_addr = end.as_ptr() as usize;

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

        self.orders[order].push(start.as_ptr() as *mut _);
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

            // `child` is the order where both of the buddies will end up in
            let child_order = order_to_split - 1;
            let child = &mut self.orders[child_order];

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
                //                       address plus the size of the order
                let buddy_addr = (block as usize) + size_for_order(child_order);

                // now insert both bodies into the child order
                child.push(block);
                child.push(buddy_addr as *mut _);
            }
        }

        // now pop one of the created buddies from the list.
        // we must have one, otherwise we either would have returned
        // already, or a buddy would've been created
        let ptr = self.orders[order]
            .pop()
            .expect("there must be a buddy available");
        let ptr = NonNull::new(ptr as *mut _).expect("heap block should never be zero");
        return Ok(ptr);
    }

    /// Deallocates a block of memory, that is located at the given pointer
    /// and was allocated in the given order.
    ///
    /// # Safety
    ///
    /// The pointer (`ptr`) must be allocated by `self` using [`alloc`](Self::alloc).
    pub unsafe fn dealloc(&mut self, ptr: NonNull<u8>, order: usize) {
        todo!()
    }
}

impl fmt::Debug for BuddyAllocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "BUDDY ALLOCATOR")?;
        writeln!(f, "~~~~~~~~~~~~~~~")?;
        for order in (0..self.orders.len()).rev() {
            let list = &self.orders[order];
            let len = list.iter().count();
            if len != 0 {
                writeln!(f, "Order {} has {} free blocks", order, len)?;
            }
        }
        writeln!(f, "~~~~~~~~~~~~~~~")?;
        Ok(())
    }
}
