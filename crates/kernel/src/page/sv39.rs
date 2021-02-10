//! Implementation of the Sv39 addressing mode

use super::{PageSize, Perm, PhysAddr, VirtAddr};

/// The central page table structure.
#[repr(transparent)]
pub struct Table {
    _entries: [Entry; 512],
}

impl Table {
    pub fn map(&mut self, paddr: PhysAddr, vaddr: VirtAddr, _size: PageSize) {
        let _vpn = vpns_of_vaddr(vaddr);
        let _ppn = ppns_of_paddr(paddr);

        todo!()
    }
}

/// A page-table entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Entry(u64);

impl Entry {
    /// Check the `V` bit of this PTE.
    #[inline]
    pub fn valid(self) -> bool {
        self.0 & 0x01 != 0
    }

    /// Check if this PTE is a leaf.
    #[inline]
    pub fn leaf(self) -> bool {
        self.perm() == 0.into()
    }

    /// Return the permissions for this PTE.
    #[inline]
    pub fn perm(self) -> Perm {
        let perm = (self.0 >> 1) & 0b111;
        Perm::from(perm as u8)
    }

    /// Check if this PTE is accessible from U-Mode.
    #[inline]
    pub fn user(self) -> bool {
        self.0 & (1 << 4) != 0
    }

    /// Check if this PTE is global mapping.
    #[inline]
    pub fn global(self) -> bool {
        self.0 & (1 << 5) != 0
    }

    /// Check if the `A` bit of this PTE is high.
    #[inline]
    pub fn access(self) -> bool {
        self.0 & (1 << 6) != 0
    }

    /// Check if the `D` bit of this PTE is high.
    #[inline]
    pub fn dirty(self) -> bool {
        self.0 & (1 << 7) != 0
    }
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

fn ppns_of_paddr(paddr: PhysAddr) -> [usize; 3] {
    let paddr = usize::from(paddr);
    [
        (paddr >> 12) & 0x1FF,
        (paddr >> 21) & 0x1FF,
        (paddr >> 30) & 0xFFFF,
    ]
}
