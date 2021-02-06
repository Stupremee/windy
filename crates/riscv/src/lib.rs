//! Common components that are used across multiple crates
//! of windy.
#![deny(rust_2018_idioms, broken_intra_doc_links)]
#![no_std]
#![feature(asm, extended_key_value_attributes, int_bits_const)]

mod bitfield;
pub use bitfield::BitField;

pub mod registers;
pub mod symbols;
pub mod sync;
pub mod trap;
