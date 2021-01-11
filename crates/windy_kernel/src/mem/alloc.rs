//! Memory Allocation APIs.

pub mod buddy;
pub mod slab;

mod linked_list;
pub use linked_list::LinkedList;

use crate::unit::KIB;
use core::fmt;

/// The size of a single page in memory.
pub const PAGE_SIZE: usize = 4 * KIB;

/// Result for every memory allocation operation.
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// Any error that can happen while allocating or deallocating memory.
#[derive(Debug)]
pub enum Error {
    /// Tried to add a region to an allocator that was too small.
    RegionTooSmall,
    /// The `end` pointer of a memory region was before the `start` pointer.
    InvalidRegion,
    /// Tried to allocate an order that exceeded the maximum order.
    OrderTooLarge,
    /// Tried to allocate, but there was no free memory left.
    NoMemoryAvailable,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::RegionTooSmall => f.write_str("memory region that was to small"),
            Error::InvalidRegion => f.write_str("region end pointer was before the start pointer"),
            Error::OrderTooLarge => f.write_str("the order that exceeded the maximum"),
            Error::NoMemoryAvailable => f.write_str("there's no free memory left"),
        }
    }
}

/// Statistics for a memory allocator.
#[derive(Debug, Clone)]
pub struct AllocStats {
    /// The name of the allocator that collected these stat.s
    pub name: &'static str,
    /// The number of size that were requested by the kernel
    pub requested: usize,
    /// The number of bytes that were actually allocated.
    pub allocated: usize,
    /// The total number of bytes that this allocator has available for allocation.
    pub total: usize,
}

impl AllocStats {
    /// Create a new [`AllocStats`] instance for the given allocator name.
    pub const fn with_name(name: &'static str) -> Self {
        Self {
            name,
            requested: 0,
            allocated: 0,
            total: 0,
        }
    }
}

impl fmt::Display for AllocStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.name)?;
        self.name.chars().try_for_each(|_| write!(f, "~"))?;
        writeln!(f, "\nRequested bytes: 0x{:x}", self.requested)?;
        writeln!(f, "Allocated bytes: 0x{:x}", self.allocated)?;
        writeln!(f, "Total bytes:     0x{:x}", self.total)?;
        self.name.chars().try_for_each(|_| write!(f, "~"))?;
        writeln!(f)?;
        Ok(())
    }
}

/// Aligns the given `addr` upwards to `align`.
///
/// # Safety
/// Requires `align` to be a power of two.
pub unsafe fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
