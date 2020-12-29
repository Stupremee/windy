#![deny(rust_2018_idioms, broken_intra_doc_links)]
#![no_std]

mod byteorder;
pub use byteorder::BigEndian;

use core::marker::PhantomData;
use cstr_core::CStr;

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
pub struct DeviceTree {
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
}

impl DeviceTree {
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
    pub unsafe fn new<'ptr>(ptr: *const u8) -> Result<&'ptr Self, Error> {
        let tree = &*ptr.cast::<DeviceTree>();
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
    pub fn memory_reservation<'tree>(&'tree self) -> &'tree [ReserveEntry<'tree>] {
        // go the first memory reservation block using
        // the offset in the header.
        let offset = self.off_mem_rsvmap.get();
        let ptr = self
            .offset_ptr(offset as usize)
            .cast::<ReserveEntry<'tree>>();

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

    /// Returns an iterator over the whole list of strings.
    pub fn strings<'tree>(&'tree self) -> Strings<'tree> {
        Strings {
            tree: self,
            ptr: self.offset_ptr(self.off_dt_strings.get() as usize),
        }
    }

    fn offset_ptr(&self, offset: usize) -> *const u8 {
        unsafe { self.as_ptr().cast::<u8>().add(offset) }
    }

    /// Returns a raw pointer to `self`.
    fn as_ptr(&self) -> *const Self {
        self as *const _
    }
}

#[repr(C)]
pub struct ReserveEntry<'tree> {
    /// The physical start address of this reservation.
    address: BigEndian<u64>,
    /// The size of the memory reservation region.
    size: BigEndian<u64>,
    _lifetime: PhantomData<&'tree ()>,
}

impl ReserveEntry<'_> {
    /// Return the address of this memory reservation.
    pub fn address(&self) -> u64 {
        self.address.get()
    }

    /// Return the size of this memory reservation.
    pub fn size(&self) -> u64 {
        self.size.get()
    }
}

/// An iterator over the string table inside a device tree.
pub struct Strings<'tree> {
    /// Pointer to the start of the next string.
    ptr: *const u8,
    tree: &'tree DeviceTree,
}

impl<'tree> Iterator for Strings<'tree> {
    type Item = &'tree CStr;

    fn next(&mut self) -> Option<Self::Item> {
        // calculate the end of the string list
        let offset = self.tree.off_dt_strings.get() as usize;
        let size = self.tree.size_dt_strings.get() as usize;
        let limit = unsafe { self.tree.offset_ptr(offset).add(size) };

        // we reached the end of the list of strings.
        if self.ptr >= limit {
            return None;
        }

        // create the actual string
        let cstr = unsafe { CStr::from_ptr(self.ptr) };
        // increment `ptr` so it points to the next string
        self.ptr = unsafe { self.ptr.add(cstr.to_bytes().len() + 1) };
        Some(cstr)
    }
}
