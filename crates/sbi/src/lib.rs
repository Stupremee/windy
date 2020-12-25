//! Custom implementation of the [OpenSBI] specification.
//!
//! This crate can be used as the software that is running in M-mode
//! and provides the SBI, and it can be used as the API for accessing
//! SBI functions.
//!
//! [OpenSBI]: https://github.com/riscv/riscv-sbi-doc
#![deny(rust_2018_idioms, broken_intra_doc_links)]
#![no_std]
#![feature(global_asm, cfg_target_has_atomic)]

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Windy can only run on 64 bit systems");

#[cfg(not(target_has_atomic = "ptr"))]
compile_error!("Windy can only run on systems that have atomic support");

mod trap_handler;

use common::registers::mtvec;

/// Initializes the `SBI` backend that will run in M-Mode.
///
/// This function will write the address of the trap handler
/// into the `mtvec` register.
///
/// # Safety
///
/// This functions has to be run in M-Mode.
pub unsafe fn init_sbi_handler() {
    let addr = trap_handler::trap_handler_address();
    mtvec::write(addr);
}
