//! Low Level Parser for parsing the structure block of a device tree.

use core::convert::TryInto;

/// Marks the beginning of a new node.
const FDT_BEGIN_NODE: u32 = 0x00000001;
/// Marks the end of a node.
const FDT_END_NODE: u32 = 0x00000002;
/// Marks the start of a new property inside a node.
const FDT_PROP: u32 = 0x00000003;
/// NOP
const FDT_NOP: u32 = 0x00000004;

/// Raw token returned by the `TokenIter`.
///
/// `FDT_NOP` tokens are automatically skipped.
#[derive(Debug, Clone)]
pub enum Token<'tree> {
    /// This token marks the beginning of a new node inside the tree.
    BeginNode(BeginNodeToken<'tree>),
    /// This token marks the beginning of a new property inside a node.
    Property(PropertyToken<'tree>),
    /// This token marks the end of a node.
    EndNode,
}

/// The data that follows a `FDT_PROP` token.
#[derive(Debug, Clone)]
pub struct PropertyToken<'tree> {
    /// A reference to the raw data of this property.
    pub data: &'tree [u8],
    /// The name of this property is at this offset inside
    /// the strings table.
    pub name_off: usize,
}

/// The data that follows a `FDT_BEGIN_NODE` token.
#[derive(Debug, Clone)]
pub struct BeginNodeToken<'tree> {
    /// The name of this begin node token.
    pub name: &'tree str,
}

/// Iterator over the raw tokens parsed from a structure
/// block inside a device tree.
#[derive(Clone)]
pub struct TokenIter<'tree> {
    buf: &'tree [u8],
}

impl<'tree> TokenIter<'tree> {
    /// Create a new `TokenIter` that can be used to further process
    /// a device tree.
    pub fn new(buf: &'tree [u8]) -> Self {
        Self { buf }
    }

    fn next_u32(&mut self) -> Option<u32> {
        if self.buf.len() < 4 {
            return None;
        }

        let (val, remaining) = self.buf.split_at(4);
        self.buf = remaining;
        Some(u32::from_be_bytes(val.try_into().ok()?))
    }
}

impl<'tree> Iterator for TokenIter<'tree> {
    type Item = Token<'tree>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_u32()?;

        match token {
            FDT_BEGIN_NODE => {
                // after a node begin token the name of the node follows as a
                // nul-terminated string
                let nul_pos = memchr::memchr(0x00, self.buf)?;
                let str_bytes = &self.buf[..nul_pos];

                // SAFETY
                // Inside a valid device tree, the name must always be UTF-8 encoded.
                let name = unsafe { core::str::from_utf8_unchecked(str_bytes) };

                // the nul-terminated string may be followed by padding to align
                // to a 4 byte boundary
                let len = crate::align_up(str_bytes.len() + 1, 4);
                self.buf = &self.buf[len..];

                Some(Token::BeginNode(BeginNodeToken { name }))
            }
            FDT_PROP => {
                // the property token is followed by the length of the data and
                // the offset of the name of this property inside the strings table.
                let len = self.next_u32()? as usize;
                let name_off = self.next_u32()? as usize;

                // after the property header comes the data that is `len` bytes
                // large and again an optional padding to align to 4 bytes
                let data = &self.buf[..len];
                let len = crate::align_up(len, 4);
                self.buf = &self.buf[len..];

                Some(Token::Property(PropertyToken { data, name_off }))
            }
            FDT_END_NODE => Some(Token::EndNode),
            // NOP tokens are skipped silently
            FDT_NOP => self.next(),
            _ => None,
        }
    }
}
