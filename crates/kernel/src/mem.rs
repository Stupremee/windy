mod rangeset;
pub use rangeset::{Error as RangeError, Range, RangeSet};

mod linked_list;
pub use linked_list::{IterMut, LinkedList, ListNode};

pub mod alloc;
pub use self::alloc::Error as AllocError;

use core::{array, ptr::NonNull};
use devicetree::DeviceTree;

displaydoc_lite::displaydoc! {
    /// Errors that are related to memory management.
    #[derive(Debug)]
    pub enum Error {
        /// {_0}
        RangeSet(RangeError),
        /// {_0}
        Alloc(AllocError),
        /// tried to add a memory region that starts at `0`.
        NullRegion,
    }
}

/// Initialize the global memory allocator.
pub unsafe fn init(tree: &DeviceTree<'_>) -> Result<(), Error> {
    let mut memory = RangeSet::new();

    tree.memory()
        .regions()
        .try_for_each(|region| {
            let range = Range::new(region.start(), region.end());
            memory.insert(range)?;
            Ok::<(), RangeError>(())
        })
        .map_err(Error::RangeSet)?;

    array::IntoIter::new(get_blocked_ranges())
        .try_for_each(|range| {
            memory.remove_range(range)?;
            Ok(())
        })
        .map_err(Error::RangeSet)?;

    memory
        .as_slice()
        .iter()
        .try_for_each(|&Range { start, end }| {
            crate::debug!(
                "Making region {:#X}..{:#X} available for allocation",
                start,
                end
            );

            let start = NonNull::new(start as *mut _).ok_or(Error::NullRegion)?;
            let end = NonNull::new(end as *mut _).ok_or(Error::NullRegion)?;
            let bytes = alloc::allocator()
                .add_region(start, end)
                .map_err(Error::Alloc)?;

            crate::debug!(
                "Made {} available for allocation",
                crate::unit::bytes(bytes)
            );

            Ok::<(), Error>(())
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
