//! Implementation of a Buddy Allocator that is responsible for allocating
//! the physical memory that will then be used by either the slab allocator
//! to allocate objects, or directly by the kernel.

use super::linked_list::LinkedList;

/// The maximum order for the buddy allocator (inclusive).
pub const MAX_ORDER: usize = 14;

/// The size of the orders array inside the buddy allocator.
pub const ORDER_COUNT: usize = MAX_ORDER + 1;

/// The central structure that is responsible for allocating
/// memory using the buddy allocation algorithm.
pub struct BuddyAllocator {
    orders: [LinkedList; ORDER_COUNT],
}
