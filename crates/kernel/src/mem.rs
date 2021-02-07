mod rangeset;
pub use rangeset::{Error as RangeError, Range, RangeSet};

mod linked_list;
pub use linked_list::{IterMut, LinkedList, ListNode};

pub mod alloc;
pub use alloc::Error as AllocError;

use core::array;
use devicetree::DeviceTree;

/// Errors that are related to memory management.
pub enum Error {
    RangeSet(RangeError),
    Alloc(AllocError),
}

/// Initialize the global memory allocator.
pub fn init(tree: &DeviceTree<'_>) -> Result<(), RangeError> {
    let mut memory = RangeSet::new();

    tree.memory().regions().try_for_each(|region| {
        let range = Range::new(region.start(), region.end());
        memory.insert(range)?;
        Ok::<(), RangeError>(())
    })?;

    array::IntoIter::new(get_blocked_ranges()).try_for_each(|range| {
        memory.remove_range(range)?;
        Ok(())
    })?;

    Ok(())
}

/// Get a list of memory ranges that must not be used for memory allocation,
/// like the kernel itself and OpenSBI.
fn get_blocked_ranges() -> [Range; 2] {
    let (kernel_start, kernel_end) = riscv::symbols::kernel_range();

    [
        // this range contains the OpenSBI firmware
        Range::new(0x8000_0000, 0x801F_FFFF),
        // the kernel itself
        Range::new(kernel_start as _, kernel_end as usize - 1),
    ]
}
