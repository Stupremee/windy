//! Custom implementation of the [OpenSBI] specification.
//!
//! This crate can be used as the software that is running in M-mode
//! and provides the SBI, and it can be used as the API for accessing
//! SBI functions.
//!
//! [OpenSBI]: https://github.com/riscv/riscv-sbi-doc
#![deny(rust_2018_idioms, broken_intra_doc_links)]
#![no_std]
#![feature(asm, global_asm, cfg_target_has_atomic)]

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Windy can only run on 64 bit systems");

#[cfg(not(target_has_atomic = "ptr"))]
compile_error!("Windy can only run on systems that have atomic support");

// mod hart_mask;
mod interface;
mod trap_handler;

pub mod platform;
pub use platform::Platform;

pub use interface::{
    SBI_IMPLEMENTATION_ID, SBI_IMPLEMENTATION_VERSION, SBI_SPEC_MAJOR, SBI_SPEC_MINOR,
    SBI_SPEC_VERSION, SUPPORTED_EXTENSIONS,
};

use windy_riscv::{
    prelude::*,
    registers::{
        mepc, mhartid,
        mstatus::{self, MSTATUS},
        mtvec,
    },
};

/// The result of a SBI call.
pub type SbiResult<T> = core::result::Result<T, Error>;

/// Initializes the `SBI` backend that will run in M-Mode.
///
/// This function will write the address of the trap handler
/// into the `mtvec` register. It will also set a global
/// [`Platform`] instance, which can not be replaced, even if you call
/// this function again.
///
/// Just call this function at the start of one your harts.
///
/// # Safety
///
/// This functions has to be run in M-Mode.
pub unsafe fn install_sbi_handler(platform: Platform) {
    let global = platform::global();

    let mut guard = global.lock();
    if guard.is_none() {
        *guard = Some(platform);
    }

    let addr = trap_handler::trap_handler_address();
    mtvec::write(addr);
}

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

    /// Reads the error code from `a0` and the value from `a1`,
    /// checks if an error occurred, and returns either `Ok(value)` or the error.
    ///
    /// # Safety
    ///
    /// This function must be called after a SBI call.
    pub unsafe fn from_sbi_call() -> SbiResult<usize> {
        let (value, err_code);
        asm!("mv {}, a0", "mv {}, a1", out(reg) err_code, out(reg) value);

        match err_code {
            0 => Ok(value),
            code => Err(Error::from_code(code)),
        }
    }
}

/// Pointer to a function that can be used as the entrypoint when
/// entering another privilege mode using [`enter_privileged`].
///
/// It takes the `hart_id` and a pointer to the device tree as arguments.
pub type PrivilegeEntry = fn(usize, *const u8) -> !;

/// Represents any privilige mode, that can be jumped into using
/// [`enter_privileged`].
#[derive(Debug, Clone, Copy)]
pub enum PrivilegeMode {
    Machine,
    Supervisor,
    User,
}

/// Enter a lower privilege mode from M-Mode.
///
/// This must be called from every hart that is supposed to enter lower privilege mode.
pub unsafe fn enter_privileged(
    mode: PrivilegeMode,
    entry: PrivilegeEntry,
    device_tree: *const u8,
) -> ! {
    // write the address of the entrypoint into `mepc`,
    // so `mret` knows where to jump to
    mepc::write(entry as _);

    // write the target privilege mode into `mstatus`.
    let mstatus = mstatus::mstatus();
    match mode {
        PrivilegeMode::Machine => mstatus.modify(MSTATUS::MPP::M),
        PrivilegeMode::Supervisor => mstatus.modify(MSTATUS::MPP::S),
        PrivilegeMode::User => mstatus.modify(MSTATUS::MPP::U),
    }

    // read hart id which will be given as an argument to the entry point
    let hart_id = mhartid::read();

    // now execute the `mret` instruction and set the argument registers
    asm!("mret", in("a0") hart_id, in("a1") device_tree, options(noreturn))
}
