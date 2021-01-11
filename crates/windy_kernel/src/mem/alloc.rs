//! Memory Allocation APIs.

pub mod buddy;
pub use buddy::BuddyAllocator;

mod slab;
pub use slab::Slab;

mod linked_list;
pub use linked_list::{IterMut, LinkedList, ListNode};

use crate::unit::{ByteUnit, KIB};
use core::{
    fmt,
    mem::{self, MaybeUninit},
    ops::Range,
    ptr::NonNull,
};

/// The size of a single page in memory.
///
/// This is also used as the order-0 size inside
/// the buddy allocator.
pub const PAGE_SIZE: usize = 4 * KIB;

/// The number of slabs used by the [`GlobalAllocator`].
const NUM_SLABS: usize = 8;

/// The order that will be allocated for each single slab.
///
/// Order `11` is the same as 8MiB.
const ORDER_PER_SLAB: usize = 11;

/// The log2 size of the first slab, that is used as the base for every other slab.
const FIRST_SLAB_SIZE: usize = 6;

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

/// The global allocator which must be used as the global
/// allocator inside the kernel.
///
/// It consists of a number of slabs that are used
/// for fixed size objects, and a fallback allocator ([`BuddyAllocator`])
/// which will be used for larger allocations.
pub struct GlobalAllocator {
    slabs: [Slab; NUM_SLABS],
    fallback: BuddyAllocator,
    stats: AllocStats,
}

impl GlobalAllocator {
    /// Creates a new, uninitialized [`GlobalAllocator`].
    pub const fn new() -> Self {
        // initialize the slabs.
        // the slabs are initialised by looping through every single slab,
        // and assign it to a new slab with the block size `1 << (idx + 6)`
        let mut slabs: [MaybeUninit<Slab>; NUM_SLABS] = MaybeUninit::uninit_array();

        let mut idx = 0usize;
        while idx < NUM_SLABS {
            let block_size = 1 << (idx + FIRST_SLAB_SIZE);
            let slab = unsafe { Slab::new(block_size) };
            slabs[idx] = MaybeUninit::new(slab);

            idx += 1;
        }

        Self {
            slabs: unsafe { mem::transmute(slabs) },
            fallback: BuddyAllocator::new(),
            stats: AllocStats::with_name("Global Allocator"),
        }
    }

    /// Initializes this global allocator by adding a single heap region.
    ///
    /// This method will initialize all allocators, including all slabs.
    /// Do not use this method for adding additional memory regions, use [`Self::add_region`]
    /// instead.
    ///
    /// # Safety
    ///
    /// The memory region must be valid to write for the lifetime of this allocator.
    ///
    /// This method **must** only be called once.
    pub unsafe fn init(&mut self, start: NonNull<u8>, end: NonNull<u8>) -> Result<()> {
        self.stats.total += self.fallback.add_region(start, end)?;

        for slab in self.slabs.iter_mut() {
            let mut region = self.fallback.allocate(ORDER_PER_SLAB)?;
            let Range { start, end } = region.as_mut().as_mut_ptr_range();
            slab.add_region(NonNull::new_unchecked(start), NonNull::new_unchecked(end));
        }

        Ok(())
    }

    /// Adds a single region of memory to the fallback allocator of this global allocator.
    pub unsafe fn add_region(&mut self, start: NonNull<u8>, end: NonNull<u8>) -> Result<()> {
        self.stats.total += self.fallback.add_region(start, end)?;
        Ok(())
    }

    /// Returns a copy of the statistics for this allocator.
    pub fn stats(&self) -> AllocStats {
        self.stats.clone()
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
        writeln!(f, "\nRequested bytes: {}", ByteUnit(self.requested))?;
        writeln!(f, "Allocated bytes:   {}", ByteUnit(self.allocated))?;
        writeln!(f, "Total bytes:       {}", ByteUnit(self.total))?;
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
