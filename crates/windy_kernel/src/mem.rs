//! Implementation of the Memory Allocation System for the kernel.

pub mod buddy;
pub mod linked_list;

use core::fmt;

/// Statistics for a memory allocator.
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
    pub fn with_name(name: &'static str) -> Self {
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
