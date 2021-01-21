//! Custom implementation of the [OpenSBI] specification.
//!
//! This crate can be used as the software that is running in M-mode
//! and provides the SBI, and it can be used as the API for accessing
//! SBI functions.
//!
//! [OpenSBI]: https://github.com/riscv/riscv-sbi-doc
#![deny(rust_2018_idioms, broken_intra_doc_links)]
#![no_std]
#![feature(asm, cfg_target_has_atomic, never_type)]

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Windy can only run on 64 bit systems");

#[cfg(not(target_has_atomic = "ptr"))]
compile_error!("Windy can only run on systems that have atomic support");

// mod hart_mask;
// mod interface;
// pub use interface::ecall;

// pub mod platform;
// pub use platform::Platform;

pub mod base;
pub mod hsm;
pub mod ipi;
pub mod system;
pub mod timer;

/// The result of a SBI call.
pub type SbiResult<T> = core::result::Result<T, Error>;

/// Standard SBI errors that can occurr while executing a
/// SBI call.
///
/// An [`Error`] is retrieved by reading the `a0` register after making
/// a SBI call. If the register is `0`, the call was successful and there's
/// probably a value available in `a1`, otherwise the SBI call failed.
#[derive(Debug, Clone)]
#[repr(usize)]
pub enum Error {
    /// The SBI call failed to execute.
    Failed,
    /// The SBI call is not supported.
    NotSupported,
    /// An invalid paramter was passed.
    InvalidParam,
    /// Denied the execution of the SBI call.
    Denied,
    /// Provided an invalid address.
    InvalidAddress,
    /// The resource is already available.
    AlreadyAvailable,
    /// Unknown SBI error was returned.
    Unknown(isize),
}

impl Error {
    /// This method tries to convert the given code into an [`Error`].
    ///
    /// Returns `None` if the provided code is invalid.
    pub fn from_code(code: isize) -> Self {
        match code {
            -1 => Error::Failed,
            -2 => Error::NotSupported,
            -3 => Error::InvalidParam,
            -4 => Error::Denied,
            -5 => Error::InvalidAddress,
            -6 => Error::AlreadyAvailable,
            code => Error::Unknown(code),
        }
    }

    /// Converts this `Error` into it's specific error code.
    pub fn code(&self) -> isize {
        match *self {
            Error::Failed => -1,
            Error::NotSupported => -2,
            Error::InvalidParam => -3,
            Error::Denied => -4,
            Error::InvalidAddress => -5,
            Error::AlreadyAvailable => -6,
            Error::Unknown(code) => code,
        }
    }

    /// Checks if the `err_code` is `0`, which is successful and thus returns `Ok(value)`,
    /// otherwise the specified error will be returned.
    pub fn from_sbi_call<T>(value: T, err_code: isize) -> SbiResult<T> {
        match err_code {
            0 => Ok(value),
            code => Err(Error::from_code(code)),
        }
    }
}
