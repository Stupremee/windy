use crate::{
    parse::{Token, TokenIter},
    DeviceTree,
};
use core::{convert::TryInto, iter::Fuse, num::NonZeroU32, slice};

/// A node inside a device tree.
#[derive(Clone)]
pub struct Node<'tree> {
    pub(crate) tree: &'tree DeviceTree<'tree>,
    pub(crate) name: &'tree str,
    /// The level inside the device tree.
    pub(crate) level: u8,
    /// A token iter that starts after the begin node token of this node.
    pub(crate) children: TokenIter<'tree>,
}

impl<'tree> Node<'tree> {
    /// The name of this node.
    pub fn name(&self) -> &'tree str {
        self.name
    }

    /// The level of this node inside the device tree.
    pub fn level(&self) -> u8 {
        self.level
    }

    /// Returns an iterator over all children nodes of this node.
    pub fn children(&self) -> Fuse<Children<'tree>> {
        Children {
            tree: self.tree,
            iter: self.children.clone(),
            node_level: self.level,
            nesting_level: self.level + 1,
        }
        .fuse()
    }

    /// Try to find a property inside this node with the given name.
    pub fn prop(&self, name: &str) -> Option<Property<'tree>> {
        self.props().find(|prop| prop.name() == name)
    }

    /// Returns an iterator over all properties of this node
    pub fn props(&self) -> Fuse<Properties<'tree>> {
        Properties {
            tree: self.tree,
            iter: self.children.clone(),
        }
        .fuse()
    }
}

/// A property of a [`Node`].
pub struct Property<'tree> {
    name: &'tree str,
    data: &'tree [u8],
}

impl<'tree> Property<'tree> {
    /// Returns the name of this property.
    pub fn name(&self) -> &'tree str {
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
    pub fn as_str(&self) -> Option<&'tree str> {
        crate::next_str_checked(self.data)
    }

    /// Returns an iterator that will try to interpret the data of this property
    /// as a list of strings.
    pub fn as_strings(&self) -> Strings<'tree> {
        Strings { table: self.data }
    }

    /// Try to interpret the data of this property as a `PHandle`.
    pub fn as_phandle(&self) -> Option<crate::PHandle> {
        self.as_u32().map(Into::into)
    }
}

/// An iterator over all children nodes of a single node.
pub struct Children<'tree> {
    tree: &'tree DeviceTree<'tree>,
    iter: TokenIter<'tree>,
    node_level: u8,
    nesting_level: u8,
}

impl<'tree> Iterator for Children<'tree> {
    type Item = Node<'tree>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.iter.next()?;
        match token {
            Token::BeginNode(node) => {
                let level = self.nesting_level;
                self.nesting_level += 1;

                // if we have a node at the next level after our
                // parent, we found a child
                if level == self.node_level + 1 {
                    Some(Node {
                        tree: self.tree,
                        children: self.iter.clone(),
                        name: node.name,
                        level,
                    })
                } else {
                    // otherwise we haven't found one and we can just continue
                    self.next()
                }
            }
            Token::EndNode => {
                self.nesting_level -= 1;

                // if we reached the end node token of our parent node,
                // we have finished
                if self.nesting_level <= self.node_level {
                    None
                } else {
                    self.next()
                }
            }
            // we don't care about properties here
            Token::Property(_) => self.next(),
        }
    }
}

/// An iterator over all properties of a single node.
pub struct Properties<'tree> {
    tree: &'tree DeviceTree<'tree>,
    iter: TokenIter<'tree>,
}

impl<'tree> Iterator for Properties<'tree> {
    type Item = Property<'tree>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.iter.next()?;
        match token {
            Token::Property(prop) => {
                // the name is inside the strings table at the offset that
                // is inside the property header.

                // SAFETY
                // The name offset inside a property header _must_ be valid otherwise
                // the device tree is invalid.
                let name = unsafe { self.tree.string_at(prop.name_off)? };

                Some(Property {
                    data: prop.data,
                    name,
                })
            }
            // if we parse the properties and we encounter a new node,
            // we already parsed all properties becauuse they _must_ be
            // before any children nodes
            Token::BeginNode(_) | Token::EndNode => None,
        }
    }
}

/// An iterator over all the strings inside the string table.
pub struct Strings<'tree> {
    table: &'tree [u8],
}

impl<'tree> Iterator for Strings<'tree> {
    type Item = &'tree str;

    fn next(&mut self) -> Option<Self::Item> {
        let string = crate::next_str_checked(self.table)?;

        // move buffer to the start of the next string, the `+ 1`
        // is required because `.to_bytes()` will not include the nul-terminator.
        self.table = &self.table[string.len() + 1..];

        // return the string
        Some(string)
    }
}

/// The `/memory` node inside a device tree
#[derive(Clone)]
pub struct MemoryNode<'tree> {
    pub(super) tree: &'tree DeviceTree<'tree>,
    pub(super) node: Node<'tree>,
}

impl<'tree> MemoryNode<'tree> {
    /// A copy of the underyling [`Node`] of this memory node.
    pub fn node(&self) -> Node<'tree> {
        self.node.clone()
    }

    /// Returns an iterator over all memory regions that are specified
    /// in this memory node.
    pub fn regions(&self) -> MemoryRegions<'tree> {
        let address_cells = self.tree.root().prop("#address-cells");
        let size_cells = self.tree.root().prop("#size-cells");
        let data = self
            .node
            .prop("reg")
            .map(|prop| prop.as_bytes())
            .unwrap_or_default();

        MemoryRegions {
            address_cells: address_cells.and_then(|prop| NonZeroU32::new(prop.as_u32()?)),
            size_cells: size_cells.and_then(|prop| NonZeroU32::new(prop.as_u32()?)),
            data,
        }
    }
}

/// Iterator over all regions of a [`MemoryNode`].
pub struct MemoryRegions<'tree> {
    address_cells: Option<NonZeroU32>,
    size_cells: Option<NonZeroU32>,
    data: &'tree [u8],
}

impl<'tree> MemoryRegions<'tree> {
    fn read<const N: usize>(&mut self) -> Option<[u8; N]> {
        let bytes = self.data.get(..N)?;
        self.data = &self.data[N..];
        Some(bytes.try_into().ok()?)
    }

    fn read_num(&mut self, count: NonZeroU32) -> Option<usize> {
        match count.get() {
            1 => Some(u32::from_be_bytes(self.read()?) as usize),
            2 => Some(u64::from_be_bytes(self.read()?) as usize),
            _ => panic!("the address cells or size cells field must be `2` or `4`"),
        }
    }
}

/// A single region of memory.
#[derive(Clone, Debug)]
pub struct MemoryRegion {
    start: usize,
    size: usize,
}

impl MemoryRegion {
    /// Return the start address of this memory region.
    pub fn start(&self) -> usize {
        self.start
    }

    /// Return the end address of this memory region.
    pub fn end(&self) -> usize {
        self.start() + self.size()
    }

    /// Return the size of this memory region.
    pub fn size(&self) -> usize {
        self.size
    }
}

impl Iterator for MemoryRegions<'_> {
    type Item = MemoryRegion;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.read_num(self.address_cells?)?;
        let size = self.read_num(self.size_cells?)?;
        Some(MemoryRegion { start, size })
    }
}
