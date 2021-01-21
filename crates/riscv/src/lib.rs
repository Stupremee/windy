//! Common components that are used across multiple crates
//! of windy.
#![deny(rust_2018_idioms, broken_intra_doc_links)]
#![no_std]
#![feature(asm, cfg_target_has_atomic)]

pub mod registers;
pub mod symbols;
pub mod trap;
