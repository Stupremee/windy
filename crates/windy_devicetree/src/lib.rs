#![deny(rust_2018_idioms, broken_intra_doc_links)]
#![no_std]

pub mod parse;

mod tree;
pub use tree::*;

use cstr_core::CStr;

///  A phandle is a way to reference another node in the devicetree.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PHandle(u32);

impl From<u32> for PHandle {
    fn from(x: u32) -> Self {
        Self(x)
    }
}

impl Into<u32> for PHandle {
    fn into(self) -> u32 {
        self.0
    }
}

/// Returns a reference to the next nul-terminated string
/// inside the buffer.
pub(crate) fn next_cstr_from_bytes(buf: &[u8]) -> Option<&CStr> {
    // find the nul-terminator and get the bytes until the terminator
    let nul_pos = memchr::memchr(0, buf)?;
    let str_bytes = buf.get(..=nul_pos)?;

    // SAFETY
    // we manually check if the bytes are nul-terminated
    Some(unsafe { CStr::from_bytes_with_nul_unchecked(str_bytes) })
}
