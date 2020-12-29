#![deny(rust_2018_idioms, broken_intra_doc_links)]
#![no_std]

mod byteorder;
pub use byteorder::BigEndian;

use core::marker::PhantomData;

/// The magic number that is at the start of every device tree.
const MAGIC: u32 = 0xD00DFEED;

#[derive(Debug)]
pub enum Error {
    /// The magic number of the device tree is wrong.
    InvalidMagic,
}

/// A flattened device tree.
#[derive(Debug)]
#[repr(C)]
pub struct DeviceTree<'ptr> {
    /// This field must contain the value `0xD00DFEED`.
    magic: BigEndian<u32>,
    /// The total size of the flattened device tree in bytes.
    total_size: BigEndian<u32>,
    /// The offset in bytes of the structure block, starting from the
    /// beginning of the device tree.
    off_dt_struct: BigEndian<u32>,
    /// The offset in bytes of the strings block, starting from the
    /// beginning of the device tree.
    off_dt_strings: BigEndian<u32>,
    /// The offset in bytes of the memory reservation block, starting from the
    /// beginning of the device tree.
    off_mem_rsvmap: BigEndian<u32>,
    /// The version of the flattened device tree structure.
    ///
    /// Must be `17` in this implementation.
    version: BigEndian<u32>,
    /// Contains the lowest version of the device tree with which the
    /// version is compatible.
    last_comp_version: BigEndian<u32>,
    /// The physical ID of the system's booting CPU.
    boot_cpuid_phys: BigEndian<u32>,
    /// The size in bytes of the strings block.
    size_dt_strings: BigEndian<u32>,
    /// The size in bytes of the structure block.
    size_dt_struct: BigEndian<u32>,
    _lifetime: PhantomData<&'ptr ()>,
}

impl<'ptr> DeviceTree<'ptr> {
    /// Converts a raw pointer into a device tree.
    ///
    /// This function will not perform check any validity checks
    /// on the given pointer.
    ///
    /// # Safety
    ///
    /// The pointer must be valid.
    /// The pointer must be a valid device tree representation.
    /// The pointer must live as long as the returned reference is used.
    pub unsafe fn new(ptr: *const u8) -> Result<&'ptr Self, Error> {
        let tree = &*ptr.cast::<DeviceTree<'ptr>>();
        if tree.magic.get() != MAGIC {
            Err(Error::InvalidMagic)
        } else {
            Ok(tree)
        }
    }

    /// Returns a slice of all memory reservations.
    ///
    /// Every memory reservation represents a chunk of memory
    /// that should **not** be used for general memory allocation.
    pub fn memory_reservation(&self) -> &'ptr [ReserveEntry] {
        let this = self.self_ptr().cast::<u8>();
        let offset = self.off_mem_rsvmap.get();

        // go the first memory reservation block using
        // the offset in the header.
        let ptr = unsafe { this.add(offset as usize).cast::<ReserveEntry>() };

        // the list of blocks is ended by a zeroed block, so
        // we loop until the zeroed block is found.
        let mut len = 0;
        loop {
            let next = unsafe { &*ptr.add(len) };

            // found zeroed block, so break
            if next.address() == 0 && next.size() == 0 {
                break;
            }

            len += 1;
        }

        // since we know the length now, create slice to the blocks.
        unsafe { core::slice::from_raw_parts(ptr, len) }
    }

    /// Returns a raw pointer to `self`.
    fn self_ptr(&self) -> *const Self {
        self as *const _
    }
}

#[repr(C)]
pub struct ReserveEntry {
    /// The physical start address of this reservation.
    address: BigEndian<u64>,
    /// The size of the memory reservation region.
    size: BigEndian<u64>,
}

impl ReserveEntry {
    /// Return the address of this memory reservation.
    pub fn address(&self) -> u64 {
        self.address.get()
    }

    /// Return the size of this memory reservation.
    pub fn size(&self) -> u64 {
        self.size.get()
    }
}
