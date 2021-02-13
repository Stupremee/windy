//! Implementation of the paging system.

pub mod sv39;

use crate::unit;
use core::ops;
use displaydoc_lite::displaydoc;

macro_rules! addr_type {
    ($(#[$attr:meta])* $pub:vis struct $name:ident;) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(transparent)]
        $pub struct $name(usize);

        impl $name {
            /// Interpret this physical address as a pointer to a `T`.
            pub fn as_ptr<T>(self) -> *mut T {
                self.0 as *mut T
            }

            /// Calculates the wrapping offset from this physical address.
            pub fn offset(self, off: usize) -> Self {
                $name::from(self.0.wrapping_add(off))
            }
        }


        impl From<usize> for $name {
            fn from(addr: usize) -> Self {
                Self(addr)
            }
        }

        impl From<$name> for usize {
            fn from(x: $name) -> usize {
                x.0
            }
        }
    };
}

displaydoc! {
    /// Errors that are related to paging.
    #[derive(Debug)]
    pub enum Error {
        /// Tried to map an address that is not aligned to the page size.
        UnalignedAddress,
        /// Failed to allocate a new page.
        Alloc(crate::pmem::AllocError),
    }
}

addr_type! {
    /// A Virtual address
    pub struct VirtAddr;
}

addr_type! {
    /// A Physical address
    pub struct PhysAddr;
}

/// Represents the different kinds of pages that can be mapped.
#[derive(Debug, Clone, Copy)]
pub enum PageSize {
    Kilopage,
    Megapage,
    Gigapage,
}

impl PageSize {
    pub fn is_aligned(self, addr: usize) -> bool {
        let align = match self {
            PageSize::Kilopage => 4 * unit::KIB,
            PageSize::Megapage => 2 * unit::MIB,
            PageSize::Gigapage => 1 * unit::GIB,
        };

        addr % align == 0
    }
}

/// Represents the permissions of a PTE.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Perm(u8);

impl Perm {
    pub const READ: Perm = Perm(0b001);
    pub const WRITE: Perm = Perm(0b010);
    pub const EXEC: Perm = Perm(0b100);

    /// Check if this permission is readable.
    pub fn read(self) -> bool {
        self | Perm::READ != Perm::from(0u8)
    }

    /// Check if this permission is writable.
    pub fn write(self) -> bool {
        self | Perm::WRITE != Perm::from(0u8)
    }

    /// Check if this permission is executable.
    pub fn exec(self) -> bool {
        self | Perm::WRITE != Perm::from(0u8)
    }
}

impl From<usize> for Perm {
    fn from(x: usize) -> Perm {
        Perm((x & 0b111) as u8)
    }
}

impl From<Perm> for usize {
    fn from(x: Perm) -> usize {
        x.0.into()
    }
}

impl From<u8> for Perm {
    fn from(x: u8) -> Perm {
        Perm(x & 0b111)
    }
}

impl From<Perm> for u8 {
    fn from(x: Perm) -> u8 {
        x.0
    }
}

impl ops::BitOr for Perm {
    type Output = Perm;

    fn bitor(self, rhs: Perm) -> Perm {
        Perm(self.0 | rhs.0)
    }
}

impl ops::BitAnd for Perm {
    type Output = Perm;

    fn bitand(self, rhs: Perm) -> Perm {
        Perm(self.0 & rhs.0)
    }
}

impl ops::BitXor for Perm {
    type Output = Perm;

    fn bitxor(self, rhs: Perm) -> Perm {
        Perm(self.0 ^ rhs.0)
    }
}
