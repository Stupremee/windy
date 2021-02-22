//! Implementation of the Sv39 addressing mode

use super::{Error, PageSize, Perm, PhysAddr, VirtAddr};
use crate::pmem;
use core::ptr::NonNull;

/// The central page table structure.
#[repr(C, align(4096))]
pub struct Table {
    entries: [Entry; 512],
}

impl Table {
    /// Create a new, empty table.
    pub const fn new() -> Self {
        Self {
            entries: [Entry::EMPTY; 512],
        }
    }

    /// Map a page with the given page size, from the given virtual address,
    /// to the physical address. The newly mapped page will have the given permissions.
    ///
    /// Note that this method will overwrite any pre-existing mapping for the given address.
    pub fn map(
        &mut self,
        paddr: PhysAddr,
        vaddr: VirtAddr,
        size: PageSize,
        perm: Perm,
    ) -> Result<(), Error> {
        // check if the addresses are aligned
        if !size.is_aligned(paddr.into()) || !size.is_aligned(vaddr.into()) {
            return Err(Error::UnalignedAddress);
        }

        let vpn = vpns_of_vaddr(vaddr);
        let ppn = ppn_of_paddr(paddr);

        let mut entry = &mut self.entries[vpn[2]];
        let level = match size {
            PageSize::Kilopage => 0,
            PageSize::Megapage => 1,
            PageSize::Gigapage => 2,
        };

        // walk down the page table to find the matching entry
        for level in (level..2).rev() {
            if !entry.valid() {
                // allocate a new page for the page table
                let page = pmem::zalloc().map_err(Error::Alloc)?.as_mut_ptr() as usize as u64;

                // make the current `entry` point to the newly allocated `page`
                entry.set((page >> 2) | Entry::VALID);
            }

            // `entry` points to the next PTE, so extract the PPN
            // and get the entry this entry points to.
            let new_entry = ((entry.get() & !0x3FF) << 2) as *mut Entry;
            entry = unsafe { &mut *new_entry.add(vpn[level]) };
        }

        // `entry` now points to the actual entry that needs to be modified
        let new_entry = (ppn << 10) | (usize::from(perm) << 2) | Entry::VALID as usize;
        entry.set(new_entry as u64);
        Ok(())
    }

    /// Identity map the given range using `size` pages.
    pub fn identity_map(
        &mut self,
        start: PhysAddr,
        end: PhysAddr,
        perm: Perm,
        size: PageSize,
    ) -> Result<(), Error> {
        let (start, end) = (usize::from(start), usize::from(end));

        if end - start < size.size() {
            return Err(Error::RangeTooSmall);
        }

        for addr in (start..end).step_by(size.size()) {
            let vaddr = VirtAddr::from(addr);
            let paddr = PhysAddr::from(addr);
            self.map(paddr, vaddr, PageSize::Kilopage, perm)?;
        }

        Ok(())
    }

    /// Tries to unmap the given virtual address.
    ///
    /// Return `true` if the unmapping was successful.
    pub fn unmap(&mut self, vaddr: VirtAddr) -> bool {
        match self.entry_mut(vaddr) {
            Some((table, entry, size)) => {
                // clear the entry to unmap the virtaddr.
                table.entries[entry].set(0);

                // now try to free the page, in which `entry` lives, if there are
                // no other entries inside the table.
                //
                // However, we will not free the table if the entry was found
                // in the root table
                if matches!(size, PageSize::Kilopage | PageSize::Megapage)
                    && table.entries.iter().all(|entry| !entry.valid())
                {
                    let page = unsafe { NonNull::new_unchecked(self as *mut _ as *mut _) };
                    unsafe { pmem::dealloc(page) };
                }

                true
            }
            None => false,
        }
    }

    /// Try to tranlsate the given virtual address, to their physical address,
    /// as mapped inside this table.
    pub fn translate(&self, vaddr: VirtAddr) -> Option<(PhysAddr, PageSize)> {
        self.entry(vaddr).map(|(_, entry, size)| {
            // extract the offset inside the page
            let off = usize::from(vaddr) & 0xFFF;
            let ppn = entry.ppn();

            (ppn.offset(off), size)
        })
    }

    fn entry(&self, vaddr: VirtAddr) -> Option<(&Table, &Entry, PageSize)> {
        let [first, second, third] = vpns_of_vaddr(vaddr);

        let entry = &self.entries[first];
        let next = match entry.kind()? {
            EntryKind::Leaf => return Some((self, entry, PageSize::Gigapage)),
            EntryKind::Branch(next) => unsafe { &*next.as_ptr::<Table>() },
        };

        let entry = &next.entries[second];
        let next = match entry.kind()? {
            EntryKind::Leaf => return Some((next, entry, PageSize::Megapage)),
            EntryKind::Branch(next) => unsafe { &*next.as_ptr::<Table>() },
        };

        let entry = &next.entries[third];
        match entry.kind()? {
            EntryKind::Leaf => Some((next, entry, PageSize::Kilopage)),
            EntryKind::Branch(_) => None,
        }
    }

    fn entry_mut(&mut self, vaddr: VirtAddr) -> Option<(&mut Table, usize, PageSize)> {
        let [first, second, third] = vpns_of_vaddr(vaddr);

        let entry = &mut self.entries[first];
        let next = match entry.kind()? {
            EntryKind::Leaf => return Some((self, first, PageSize::Gigapage)),
            EntryKind::Branch(next) => unsafe { &mut *next.as_ptr::<Table>() },
        };

        let entry = &mut next.entries[second];
        let next = match entry.kind()? {
            EntryKind::Leaf => return Some((next, second, PageSize::Megapage)),
            EntryKind::Branch(next) => unsafe { &mut *next.as_ptr::<Table>() },
        };

        let entry = &mut next.entries[third];
        match entry.kind()? {
            EntryKind::Leaf => Some((next, third, PageSize::Kilopage)),
            EntryKind::Branch(_) => None,
        }
    }
}

/// A page-table entry.
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Entry(u64);

impl Entry {
    pub const EMPTY: Entry = Entry(0);

    /// The `V` bit inside a PTE.
    pub const VALID: u64 = 1 << 0;
    /// The `U` bit inside a PTE.
    pub const USER: u64 = 1 << 4;
    /// The `G` bit inside a PTE.
    pub const GLOBAL: u64 = 1 << 5;
    /// The `A` bit inside a PTE.
    pub const ACCSES: u64 = 1 << 6;
    /// The `D` bit inside a PTE.
    pub const DIRTY: u64 = 1 << 7;

    /// Set the raw value of this entry to the given value.
    #[inline]
    pub fn set(&mut self, x: u64) {
        self.0 = x;
    }

    /// Get the raw value of this entry.
    #[inline]
    pub fn get(&self) -> u64 {
        self.0
    }

    /// Get the kind of this entry.
    ///
    /// Returns `None` if this entry is invalid.
    pub fn kind(&self) -> Option<EntryKind> {
        match (self.valid(), self.branch()) {
            (true, true) => {
                let next = ((self.0 as usize >> 10) & 0x0FFF_FFFF_FFFF) << 12;
                let next = PhysAddr::from(next);
                Some(EntryKind::Branch(next))
            }
            (true, false) => Some(EntryKind::Leaf),
            _ => None,
        }
    }

    /// Check the `V` bit of this PTE.
    #[inline]
    pub fn valid(&self) -> bool {
        self.0 & Entry::VALID != 0
    }

    /// Check if this PTE is a branch to the next level.
    #[inline]
    pub fn branch(&self) -> bool {
        self.perm() == Perm::from(0u8)
    }

    /// Return the permissions for this PTE.
    #[inline]
    pub fn perm(&self) -> Perm {
        let perm = (self.0 >> 1) & 0b111;
        Perm::from(perm as u8)
    }

    /// Check if this PTE is accessible from U-Mode.
    #[inline]
    pub fn user(&self) -> bool {
        self.0 & Entry::USER != 0
    }

    /// Check if this PTE is global mapping.
    #[inline]
    pub fn global(&self) -> bool {
        self.0 & Entry::GLOBAL != 0
    }

    /// Check if the `A` bit of this PTE is high.
    #[inline]
    pub fn access(&self) -> bool {
        self.0 & Entry::ACCSES != 0
    }

    /// Check if the `D` bit of this PTE is high.
    #[inline]
    pub fn dirty(&self) -> bool {
        self.0 & Entry::DIRTY != 0
    }

    /// Return the physical page number for this entry.
    #[inline]
    pub fn ppn(&self) -> PhysAddr {
        PhysAddr::from((self.0 as usize >> 10) << 12)
    }
}

/// Represents the different kinds of page table entries.
#[derive(Debug, Clone)]
pub enum EntryKind {
    /// This entry points to the entry in the next level.
    Branch(PhysAddr),
    /// This entry is a leaf and can directly be used to translate an address.
    Leaf,
}

fn vpns_of_vaddr(vaddr: VirtAddr) -> [usize; 3] {
    const MASK: usize = 0x1FF;

    let vaddr = usize::from(vaddr);
    [
        (vaddr >> 12) & MASK,
        (vaddr >> 21) & MASK,
        (vaddr >> 30) & MASK,
    ]
}

fn ppn_of_paddr(paddr: PhysAddr) -> usize {
    (usize::from(paddr) >> 12) & 0x0FFF_FFFF_FFFF
}
