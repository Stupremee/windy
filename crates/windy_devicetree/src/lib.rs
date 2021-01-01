#![deny(rust_2018_idioms, broken_intra_doc_links)]
#![no_std]

pub mod parse;

mod tree;
pub use tree::*;

use cstr_core::CStr;

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
