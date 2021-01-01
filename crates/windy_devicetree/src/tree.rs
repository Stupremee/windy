//! The central structure to represent a flattened device tree.
//! Most of the operations on a tree are done using iterators.

use crate::parse::{Token, TokenIter};
use core::{convert::TryInto, num::NonZeroUsize};
use cstr_core::CStr;

const MAGIC: u32 = 0xD00DFEED;

/// Representation of a flattened device tree.
pub struct DeviceTree<'tree> {
    /// The raw data of the tree.
    buf: &'tree [u8],
}

impl<'tree> DeviceTree<'tree> {
    /// Tries to create a new `DeviceTree` from a raw pointer to the
    /// flattened device tree.
    ///
    /// # Safety
    ///
    /// - `ptr` must be valid and non-null.
    /// - `ptr` must point to a valid FTD
    /// - `ptr` must not live shorter then the `'tree` lifetime
    ///
    /// # Returns
    ///
    /// `None` if the device tree failed to verify or parse.
    pub unsafe fn from_ptr(ptr: *const u8) -> Option<DeviceTree<'tree>> {
        // read and verify the magic number
        if read_u32(ptr) != MAGIC {
            return None;
        }

        // read `totalsize` to make a slice out of the raw pointer.
        let size = read_u32(ptr.add(4));

        // create the slice and return the tree
        let buf = core::slice::from_raw_parts(ptr, size as usize);
        Some(Self { buf })
    }

    /// Returns an iterator over all nodes and properties of this device tree.
    pub fn items(&'tree self) -> Option<Items<'tree>> {
        // get the raw structure block bytes
        let start = self.struct_offset()? as usize;
        let size = self.struct_size()? as usize;
        let buf = self.buf.get(start..start + size)?;

        // create the token iterator
        let tokens = TokenIter::new(buf);

        Some(Items {
            tree: self,
            iter: tokens,
        })
    }

    /// Return an iterator over the string table.
    pub fn strings(&'tree self) -> Option<Strings<'tree>> {
        // get the raw table in bytes
        let start = self.strings_offset()? as usize;
        let size = self.strings_size()? as usize;
        let buf = self.buf.get(start..start + size)?;
        Some(Strings {
            table: buf,
            tree: self,
        })
    }

    /// Size of the strings block.
    fn strings_size(&'tree self) -> Option<u32> {
        self.u32_at(8)
    }

    /// Offset of the strings block.
    fn strings_offset(&'tree self) -> Option<u32> {
        self.u32_at(3)
    }

    /// Size of the structure block.
    fn struct_size(&'tree self) -> Option<u32> {
        self.u32_at(9)
    }

    /// Offset of the structure block.
    fn struct_offset(&'tree self) -> Option<u32> {
        self.u32_at(2)
    }

    /// Return the `idx`nth u32 inside the buffer.
    fn u32_at(&self, idx: usize) -> Option<u32> {
        let real_idx = idx * 4;
        let bytes = self.buf.get(real_idx..real_idx + 4)?;
        Some(u32::from_be_bytes(bytes.try_into().ok()?))
    }
}

/// Iterator over all nodes and properties of this device tree.
pub struct Items<'tree> {
    iter: TokenIter<'tree>,
    tree: &'tree DeviceTree<'tree>,
}

impl<'tree> Iterator for Items<'tree> {
    type Item = NodeOrProperty<'tree>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.iter.next()?;

        match token {
            Token::BeginNode(node) => {
                let node = Node { name: node.name };
                Some(NodeOrProperty::Node(node))
            }
            Token::Property(prop) => {
                // get the name of this property from the string table
                let name = self.tree.strings()?.string_at(prop.name_offset)?;

                let prop = Property { name };
                Some(NodeOrProperty::Property(prop))
            }
            Token::EndNode => self.next(),
        }
    }
}

/// Either a node or a property.
#[derive(Debug)]
pub enum NodeOrProperty<'tree> {
    Node(Node<'tree>),
    Property(Property<'tree>),
}

/// A node that is inside a device tree.
#[derive(Debug)]
pub struct Node<'tree> {
    name: &'tree CStr,
}

/// A property of a [`Node`].
#[derive(Debug)]
pub struct Property<'tree> {
    name: &'tree CStr,
}

/// An iterator over all the strings inside the string table.
pub struct Strings<'tree> {
    /// The `table` starts where the next string starts,
    /// and ends at the end of the string table.
    table: &'tree [u8],
    tree: &'tree DeviceTree<'tree>,
}

impl<'tree> Strings<'tree> {
    /// Return the `CStr` that starts at the given `offset`.
    pub fn string_at(&self, offset: usize) -> Option<&'tree CStr> {
        let string = self.table.get(offset..)?;
        crate::next_cstr_from_bytes(string)
    }
}

impl<'tree> Iterator for Strings<'tree> {
    type Item = &'tree CStr;

    fn next(&mut self) -> Option<Self::Item> {
        // get the next CStr from the inner buffer
        let string = crate::next_cstr_from_bytes(self.table)?;

        // move buffer to the start of the next string, the `+ 1`
        // is required because `.to_bytes()` will not include the nul-terminator.
        self.table = &self.table[string.to_bytes().len() + 1..];

        // return the string
        Some(string)
    }
}

/// Reads a big endian `u32` from the ptr.
unsafe fn read_u32(ptr: *const u8) -> u32 {
    let val = *ptr.cast::<u32>();
    u32::from_be(val)
}
