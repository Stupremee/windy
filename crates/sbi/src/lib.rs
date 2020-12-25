//! Custom implementation of the [OpenSBI] specification.
//!
//! This crate can be used as the software that is running in M-mode
//! and provides the SBI, and it can be used as the API for accessing
//! SBI functions.
//!
//! [OpenSBI]: https://github.com/riscv/riscv-sbi-doc
#![deny(rust_2018_idioms, broken_intra_doc_links)]
#![no_std]
