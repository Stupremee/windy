use crate::parse::{Token, TokenIter};
use core::iter::Fuse;

/// A node inside a device tree.
#[derive(Clone)]
pub struct Node<'tree> {
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
            iter: self.children.clone(),
            node_level: self.level,
            nesting_level: self.level + 1,
        }
        .fuse()
    }
}

/// An iterator over all children nodes of a single node.
pub struct Children<'tree> {
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
