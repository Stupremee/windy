//! Implementation of a Buddy Allocator that is responsible for allocating
//! the physical memory that will then be used by either the slab allocator
//! to allocate objects, or directly by the kernel.

/// The maximum order for the buddy allocator (inclusive).
pub const MAX_ORDER: usize = 14;

/// The size of the orders array inside the buddy allocator.
pub const ORDER_COUNT: usize = MAX_ORDER + 1;
