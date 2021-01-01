//! The central structure to represent a flattened device tree.
//! Most of the operations on a tree are done using iterators.

use crate::{
    parse::{Token, TokenIter},
    PHandle,
};
use core::{cell::Cell, convert::TryInto, marker::PhantomData};
use cstr_core::CStr;

const MAGIC: u32 = 0xD00DFEED;

/// Representation of a flattened device tree.
#[derive(Clone)]
pub struct DeviceTree<'tree> {
    /// The raw data of the tree.
    buf: &'tree [u8],

    // Is this really required?
    _send_sync: PhantomData<Cell<u8>>,
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
        Some(Self {
            buf,
            _send_sync: PhantomData,
        })
    }

    /// Return an iterator over all nodes that are at the given `level`
    /// inside the tree structure.
    pub fn nodes_at_level(&'tree self, level: u8) -> Option<impl Iterator<Item = Node<'tree>>> {
        Some(self.nodes()?.filter(move |node| node.level == level))
    }

    /// Tries to find a node with the given path inside this device tree.
    ///
    /// The path is a `/` separated list of node names, and must start with `/`
    /// to indicate the root node.
    ///
    /// # Examples
    /// - `/` => Root Node
    /// - `/cpus` => CPUs Node
    /// - `/cpus/cpu0` => Node of the 0th CPU
    pub fn node(&'tree self, path: &str) -> Option<Node<'tree>> {
        // path didn't start with root node
        if !path.starts_with('/') {
            return None;
        }

        // get every single part of the path.
        let mut parts = path.split_terminator('/');

        let mut current_part = parts.next()?;
        let mut current_level = 0;

        for node in self.nodes()? {
            // check if the node is at the current level,
            // and if the name of the node matches the current part
            // of the full path
            if node.level == current_level
                && node
                    .name()
                    .to_str()
                    .map_or(false, |name| name == current_part)
            {
                current_part = match parts.next() {
                    Some(part) => part,
                    // there are no parts left in the path,
                    // so we found our node
                    None => return Some(node),
                };
                current_level += 1;
            }
        }

        // no node found
        None
    }

    /// Returns an iterator over all the nodes of this tree.
    pub fn nodes(&'tree self) -> Option<impl Iterator<Item = Node<'tree>>> {
        let iter = self.items()?.filter_map(|item| match item {
            NodeOrProperty::Node(node) => Some(node),
            _ => None,
        });
        Some(iter)
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
            props_only: 0,
            level: 0,
        })
    }

    /// Return an iterator over the string table.
    pub fn strings(&'tree self) -> Option<Strings<'tree>> {
        // get the raw table in bytes
        let start = self.strings_offset()? as usize;
        let size = self.strings_size()? as usize;
        let buf = self.buf.get(start..start + size)?;
        Some(Strings { table: buf })
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
#[derive(Clone)]
pub struct Items<'tree> {
    iter: TokenIter<'tree>,
    tree: &'tree DeviceTree<'tree>,
    /// Indicates if this Items parse should stop
    /// after the first end node token and only
    /// emits the properties for one node.
    ///
    /// `0`: normal mode, emit everything
    /// `1`: only parse properties of one node
    /// `2`: stop to parse and return `None` every time
    props_only: u8,
    /// The current node level
    level: u8,
}

impl<'tree> Iterator for Items<'tree> {
    type Item = NodeOrProperty<'tree>;

    fn next(&mut self) -> Option<Self::Item> {
        // check if this iterator has stopped
        if self.props_only == 2 {
            return None;
        }

        let token = self.iter.next()?;

        match token {
            Token::BeginNode(node) => {
                // increase the node level
                let level = self.level;
                self.level += 1;

                // create a new `TokenIter` that will iterator only over the
                // properties of the new node
                let props_buf = &self.iter.buf[self.iter.offset..];
                let props = TokenIter::new(props_buf);
                let props = Items {
                    iter: props,
                    tree: self.tree,
                    props_only: 1,
                    level,
                };

                let node = Node {
                    name: node.name,
                    props: Properties { iter: props },
                    level,
                };
                Some(NodeOrProperty::Node(node))
            }
            Token::Property(prop) => {
                // get the name of this property from the string table
                let name = self.tree.strings()?.string_at(prop.name_offset)?;

                let prop = Property {
                    name,
                    data: prop.data,
                };
                Some(NodeOrProperty::Property(prop))
            }
            Token::EndNode if self.props_only == 1 => {
                self.props_only = 2;
                None
            }
            Token::EndNode => {
                self.level -= 1;
                self.next()
            }
        }
    }
}

/// Iterator over all properties of a single node.
#[derive(Clone)]
pub struct Properties<'tree> {
    iter: Items<'tree>,
}

impl<'tree> Iterator for Properties<'tree> {
    type Item = Property<'tree>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next()?;
        match next {
            NodeOrProperty::Property(prop) => Some(prop),
            NodeOrProperty::Node(_) => self.next(),
        }
    }
}

/// Either a node or a property.
pub enum NodeOrProperty<'tree> {
    Node(Node<'tree>),
    Property(Property<'tree>),
}

/// A node that is inside a device tree.
pub struct Node<'tree> {
    name: &'tree CStr,
    props: Properties<'tree>,
    /// The level of this node inside the tree.
    ///
    /// Root node is level `0`,
    /// `/cpus` is level `1`,
    /// and `/cpus/cpu0` is level `2`
    pub level: u8,
}

impl<'tree> Node<'tree> {
    /// Returns the name of this `Node`
    pub fn name(&self) -> &'tree CStr {
        self.name
    }

    /// Tries to convert the name of this `Node` into utf8 and
    /// then returns the name if successful.
    pub fn name_utf8(&self) -> Option<&'tree str> {
        self.name.to_str().ok()
    }

    /// Tries to find a property inside this `Node` that has the given name.
    pub fn prop(&self, query: &str) -> Option<Property<'tree>> {
        self.props()
            .filter(|prop| prop.name.to_str().map_or(false, |name| query == name))
            .next()
    }

    /// Return an iterator that iterates over the properties
    /// of this node.
    pub fn props(&self) -> Properties<'tree> {
        self.props.clone()
    }
}

/// A property of a [`Node`].
pub struct Property<'tree> {
    name: &'tree CStr,
    data: &'tree [u8],
}

impl<'tree> Property<'tree> {
    /// Returns the name of this property.
    pub fn name(&self) -> &'tree CStr {
        self.name
    }

    /// Returns the raw bytes of this property data.
    pub fn as_bytes(&self) -> &'tree [u8] {
        self.data
    }

    /// Try to interpret the data of this property as a big endian `u32`.
    pub fn as_u32(&self) -> Option<u32> {
        // try to read the big endian `u32` from the data
        Some(u32::from_be_bytes(self.data.try_into().ok()?))
    }

    /// Try to interpret the data of this property as a big endian `u64`.
    pub fn as_u64(&self) -> Option<u64> {
        // try to read the big endian `u64` from the data
        Some(u64::from_be_bytes(self.data.try_into().ok()?))
    }

    /// Try to interpret the data of this property as a nul-terminated string.
    pub fn as_str(&self) -> Option<&'tree CStr> {
        crate::next_cstr_from_bytes(self.data)
    }

    /// Returns an iterator that will try to interpret the data of this property
    /// as a list of strings.
    pub fn as_strings(&self) -> Strings<'tree> {
        Strings { table: self.data }
    }

    /// Try to interpret the data of this property as a `PHandle`.
    pub fn as_phandle(&self) -> Option<PHandle> {
        self.as_u32().map(Into::into)
    }
}

/// An iterator over all the strings inside the string table.
pub struct Strings<'tree> {
    /// The `table` starts where the next string starts,
    /// and ends at the end of the string table.
    table: &'tree [u8],
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
