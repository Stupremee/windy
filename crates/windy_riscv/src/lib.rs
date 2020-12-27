//! Common components that are used across multiple crates
//! of windy.
#![deny(rust_2018_idioms, broken_intra_doc_links)]
#![no_std]
#![feature(asm, cfg_target_has_atomic)]

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Windy can only run on 64 bit systems");

#[cfg(not(target_has_atomic = "ptr"))]
compile_error!("Windy can only run on systems that have atomic support");

pub mod registers;
pub mod symbols;
pub mod trap;

/// Prelude for `windy_riscv` which exports common traits,
/// like the CPU register traits from [`register`](https://docs.rs/register).
pub mod prelude {
    pub use register::cpu::*;
}
