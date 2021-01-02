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

    /// Returns the root node of this device tree
    pub fn root(&'tree self) -> Option<Node<'tree>> {
        self.nodes().next()
    }

    /// Return an iterator over all nodes that are at the given `level`
    /// inside the tree structure.
    pub fn nodes_at_level(&'tree self, level: u8) -> impl Iterator<Item = Node<'tree>> {
        self.nodes().filter(move |node| node.level == level)
    }

    /// Returns the first node that matches the given path
    pub fn find_node<'query>(&'tree self, path: &'query str) -> Option<Node<'tree>> {
        self.find_nodes(path).next()
    }

    /// Returns an iterator over all nodes that match the given path.
    ///
    /// The path is a `/` separated list of node names, and must start with `/`
    /// to indicate the root node.
    ///
    /// # Examples
    /// - `/` => Root Node
    /// - `/cpus` => CPUs Node
    /// - `/cpus/cpu0` => Node of the 0th CPU
    pub fn find_nodes<'query>(&'tree self, path: &'query str) -> FindNodes<'tree, 'query> {
        FindNodes::new(self, path)
    }

    /// Returns an iterator over all the nodes of this tree.
    pub fn nodes(&'tree self) -> Nodes<'tree> {
        Nodes { iter: self.items() }
    }

    /// Returns an iterator over all nodes and properties of this device tree.
    pub fn items(&'tree self) -> Items<'tree> {
        // get the raw structure block bytes
        let start = self.struct_offset().unwrap() as usize;
        let size = self.struct_size().unwrap() as usize;
        let buf = &self.buf[start..start + size];

        // create the token iterator
        let tokens = TokenIter::new(buf);

        Items {
            tree: self,
            iter: tokens,
            level: 0,
        }
    }

    /// Return an iterator over the string table.
    pub fn strings(&'tree self) -> Strings<'tree> {
        // get the raw table in bytes
        let start = self.strings_offset().unwrap() as usize;
        let size = self.strings_size().unwrap() as usize;
        let buf = &self.buf[start..start + size];
        Strings { table: buf }
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

/// Iterator over all nodes that were found by a search path
pub struct FindNodes<'tree, 'query> {
    last_part: &'query str,
    /// If `nodes` and `single_node` are `None`, this iterator
    /// must always reuturn `None`.
    nodes: Option<Children<'tree>>,
    single_node: Option<Node<'tree>>,
}

impl<'tree, 'query> FindNodes<'tree, 'query> {
    fn new(tree: &'tree DeviceTree<'tree>, path: &'query str) -> FindNodes<'tree, 'query> {
        match Self::new_inner(tree, path) {
            Some(iter) => iter,
            None => Self {
                last_part: path,
                nodes: None,
                single_node: None,
            },
        }
    }

    fn new_inner(
        tree: &'tree DeviceTree<'tree>,
        path: &'query str,
    ) -> Option<FindNodes<'tree, 'query>> {
        // root node has to be handled speciallly
        if path == "/" {
            return Some(Self {
                last_part: path,
                single_node: tree.nodes_at_level(0).next(),
                nodes: None,
            });
        }

        // get the parts of the search query
        let mut parts = path.split_terminator('/').peekable();
        // skip the first one because it represents the root node
        parts.next()?;

        // now get the root node and use the children of the root node as the starting point
        let root = tree.nodes_at_level(0).next()?;
        let mut children = root.children();

        // get the next part
        let mut next_part = parts.next();

        // loop until we have no parts left from the query and search the
        // node with the part inside the current `children`.
        while let Some(part) = next_part {
            // if `part` is the last part of the query,
            // we will not continue searching and instead delegete the search process
            // to the iterator implementation
            if parts.peek().is_none() {
                break;
            }

            // find the node which matches the query
            let node = children.find(|node| Self::names_equal(node.name(), part))?;
            // get the next level of children
            children = node.children();

            // get next query part
            next_part = parts.next();
        }

        Some(Self {
            nodes: Some(children),
            last_part: next_part?,
            single_node: None,
        })
    }

    fn names_equal(a: &CStr, b: &str) -> bool {
        fn inner(a: &CStr, b: &str) -> Option<bool> {
            let (a_name, a_unit) = {
                let mut parts = a.to_str().ok()?.split('@');
                (parts.next()?, parts.next())
            };
            let (b_name, b_unit) = {
                let mut parts = b.split('@');
                (parts.next()?, parts.next())
            };

            let is_name_same = a_name == b_name;
            let is_unit_same = match (a_unit, b_unit) {
                (Some(a), Some(b)) => a == b,
                (Some(_), None) => true,
                (None, Some(_)) => true,
                (None, None) => true,
            };

            Some(is_name_same && is_unit_same)
        }

        inner(a, b).unwrap_or(false)
    }
}

impl<'tree> Iterator for FindNodes<'tree, '_> {
    type Item = Node<'tree>;

    fn next(&mut self) -> Option<Self::Item> {
        let name = self.last_part;

        if let Some(ref node) = self.single_node {
            Some(node.clone())
        } else if let Some(ref mut nodes) = self.nodes {
            nodes
                .filter(|node| Self::names_equal(node.name(), name))
                .next()
        } else {
            None
        }
    }
}

/// Iterator over all nodes and properties of this device tree.
#[derive(Clone)]
pub struct Items<'tree> {
    iter: TokenIter<'tree>,
    tree: &'tree DeviceTree<'tree>,
    /// The current node level
    level: u8,
}

impl<'tree> Iterator for Items<'tree> {
    type Item = NodeOrProperty<'tree>;

    fn next(&mut self) -> Option<Self::Item> {
        // create a new `TokenIter` that will iterator only over the
        // properties of the new node, and the children of this node
        let node_buf = &self.iter.buf[self.iter.offset..];

        let token = self.iter.next()?;

        match token {
            Token::BeginNode(node) => {
                // increase the node level
                let level = self.level;
                self.level += 1;

                let node = Node {
                    tree: self.tree,
                    data: node_buf,
                    name: node.name,
                    level,
                };
                Some(NodeOrProperty::Node(node))
            }
            Token::Property(prop) => {
                // get the name of this property from the string table
                let name = self.tree.strings().string_at(prop.name_offset)?;

                let prop = Property {
                    name,
                    data: prop.data,
                };
                Some(NodeOrProperty::Property(prop))
            }
            Token::EndNode => {
                self.level -= 1;
                self.next()
            }
        }
    }
}

/// Either a node or a property.
pub enum NodeOrProperty<'tree> {
    Node(Node<'tree>),
    Property(Property<'tree>),
}

/// A node that is inside a device tree.
#[derive(Clone)]
pub struct Node<'tree> {
    tree: &'tree DeviceTree<'tree>,
    data: &'tree [u8],
    name: &'tree CStr,

    /// The level of this node inside the tree.
    ///
    /// Root node is level `0`,
    /// `/cpus` is level `1`,
    /// and `/cpus/cpu0` is level `2`
    level: u8,
}

impl<'tree> Node<'tree> {
    /// Returns the name of this `Node`
    pub fn name(&self) -> &'tree CStr {
        self.name
    }

    /// Try to find and parse a unit address of this node.
    pub fn unit_addr(&self) -> Option<usize> {
        let addr = self.name.to_str().ok()?.split('@').nth(1)?;
        addr.parse().ok()
    }

    /// Returns the level of this `Node` inside the tree
    ///
    /// `/` is level `0`,
    /// `/cpus` is level `1`,
    /// `/cpus/cpu0` is level `2`
    /// and so on...
    pub fn level(&self) -> u8 {
        self.level
    }

    /// Returns an iterator over all children of this node.
    ///
    /// Note that the iterator only iterates over the children
    /// of this node, and not the children of the children.
    pub fn children(&self) -> Children<'tree> {
        Children {
            tree: self.tree,
            iter: TokenIter::new(self.data),
            level: self.level,
            child_level: self.level + 1,
        }
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
        Properties {
            tree: self.tree,
            iter: TokenIter::new(self.data),
            level: self.level,
            node_level: self.level,
        }
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

/// An iterator over all nodes inside a tree.
pub struct Nodes<'tree> {
    iter: Items<'tree>,
}

impl<'tree> Iterator for Nodes<'tree> {
    type Item = Node<'tree>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next()?;
        match next {
            NodeOrProperty::Node(node) => Some(node),
            _ => self.next(),
        }
    }
}

/// An iterator over all children of a node.
pub struct Children<'tree> {
    tree: &'tree DeviceTree<'tree>,
    iter: TokenIter<'tree>,
    level: u8,
    /// The level of the children nodes
    child_level: u8,
}

impl<'tree> Iterator for Children<'tree> {
    type Item = Node<'tree>;

    fn next(&mut self) -> Option<Self::Item> {
        // if level is `0`, it means that this
        // iterator has finished and shouldn't return any
        // new nodes.
        if self.child_level == 0 {
            return None;
        }

        let tok = self.iter.next()?;
        match tok {
            Token::BeginNode(node) => {
                // store the current level and increase it afterwards
                let level = self.level;
                self.level += 1;

                // if the node is in the level of the children,
                // return it, otherwise go to next token
                if level == self.child_level {
                    let data = &self.iter.buf[self.iter.offset..];
                    let node = Node {
                        tree: self.tree,
                        data,
                        name: node.name,
                        level,
                    };
                    Some(node)
                } else {
                    self.next()
                }
            }
            // properties are just ignored
            Token::Property(_) => self.next(),
            Token::EndNode => {
                // if this is the end node of our parent,
                // this is true and we stop the iterator
                if self.level == self.child_level {
                    // stop the iterator
                    self.child_level = 0;
                    None
                } else {
                    self.level -= 1;
                    // otherwise we continue with the next element
                    self.next()
                }
            }
        }
    }
}

/// Iterator over all properties of a single node.
#[derive(Clone)]
pub struct Properties<'tree> {
    iter: TokenIter<'tree>,
    tree: &'tree DeviceTree<'tree>,
    level: u8,
    node_level: u8,
}

impl<'tree> Iterator for Properties<'tree> {
    type Item = Property<'tree>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next()?;
        match next {
            Token::BeginNode(_) => {
                self.level += 1;
                self.next()
            }
            Token::Property(prop) => {
                if self.node_level == self.level - 1 {
                    let name = self.tree.strings().string_at(prop.name_offset)?;
                    let prop = Property {
                        name,
                        data: prop.data,
                    };
                    Some(prop)
                } else {
                    None
                }
            }
            Token::EndNode => {
                self.level -= 1;
                self.next()
            }
        }
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
