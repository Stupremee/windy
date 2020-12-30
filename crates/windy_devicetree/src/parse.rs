//! Low Level Parsing for the Flattened Device Tree format.

use core::convert::TryInto;
use cstr_core::CStr;

/// Marks the beginning of a new node.
const FDT_BEGIN_NODE: u32 = 0x00000001;
/// Marks the end of a node.
const FDT_END_NODE: u32 = 0x00000002;
/// Marks the start of a new property inside a node.
const FDT_PROP: u32 = 0x00000003;
/// NOP
const FDT_NOP: u32 = 0x00000004;
/// Marks the end of the whole structure block.
const FDT_END: u32 = 0x00000009;

/// Raw token returned by the `TokenIter`.
///
/// `FDT_NOP` tokens are automatically skipped.
pub enum Token<'tree> {
    /// This token marks the beginning of a new node inside the tree.
    BeginNode(BeginNodeToken<'tree>),
    /// This token marks the beginning of a new property inside a node.
    Property(PropertyToken<'tree>),
    /// This token marks the end of a node.
    EndNode,
}

/// The data that follows a `FDT_PROP` token.
pub struct PropertyToken<'tree> {
    /// A reference to the raw data of this property.
    pub data: &'tree [u8],
    /// The name of this property is at this offset inside
    /// the strings table.
    pub name_offset: usize,
}

/// The data that follows a `FDT_BEGIN_NODE` token.
pub struct BeginNodeToken<'tree> {
    /// The name of this begin node token.
    pub name: &'tree CStr,
}

/// Iterator over the raw tokens parsed from a structure
/// block inside a device tree.
pub struct TokenIter<'tree> {
    buf: &'tree [u8],
    /// The current offset where this iterator is.
    offset: usize,
}

impl<'tree> TokenIter<'tree> {
    /// Create a new `TokenIter` that can be used to further process
    /// a device tree.
    pub fn new(buf: &'tree [u8]) -> Self {
        Self { buf, offset: 0 }
    }

    fn next_u32(&mut self) -> Option<u32> {
        let bytes = self.buf.get(self.offset..self.offset + 4)?;
        Some(u32::from_be_bytes(bytes.try_into().ok()?))
    }
}

impl<'tree> Iterator for TokenIter<'tree> {
    type Item = Token<'tree>;

    fn next(&mut self) -> Option<Self::Item> {
        // get the next token
        let token = self.next_u32()?;

        // increment `offset` so we point to the next `u32`
        self.offset += 4;

        match token {
            // beginning of a new node is followed by it's name
            // and optional padding for aliging to 4 bytes
            FDT_BEGIN_NODE => {
                // we manually serach for the null byte because `cstr_core` will
                // fail if the nul byte is not the last one.

                // get the raw bytes and find the position of the nul terminator
                let str_bytes = self.buf.get(self.offset..)?;
                let nul_pos = memchr::memchr(0, str_bytes)?;
                let str_bytes = str_bytes.get(..=nul_pos)?;

                // SAFETY
                // we manually check if the bytes are nul-terminated
                let name = unsafe { CStr::from_bytes_with_nul_unchecked(str_bytes) };

                // skip all bytes we have just read as the `name`
                self.offset += name.to_bytes().len() + 1;

                // there may be padding after the string to align to 4 bytes
                while self.offset % 4 != 0 {
                    self.offset += 1;
                }

                Some(Token::BeginNode(BeginNodeToken { name }))
            }

            // a property contains a header, which contains the length of the data and
            // the name offset, and after the header is the actual raw data
            FDT_PROP => {
                // get the len and offset field of the prop header
                let len = self.next_u32()?;
                let name_offset = self.next_u32()? as usize;

                // get a slice to the data of this property
                let data = self.buf.get(self.offset..self.offset + len as usize)?;

                // there may be padding after the data to align to 4 bytes
                while self.offset % 4 != 0 {
                    self.offset += 1;
                }

                Some(Token::Property(PropertyToken { data, name_offset }))
            }

            // end node token can be returned as it is.
            FDT_END_NODE => Some(Token::EndNode),

            // if we have found `NOP`, skip it and recursively
            // look for the next token
            FDT_NOP => self.next(),

            // we either found an invalid token,
            // or reached the end of the structure block.
            FDT_END => None,
            _ => None,
        }
    }
}
