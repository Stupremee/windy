//! Implementation of the SBI extensions and the actual SBI
//! method execution.

use super::{Error, SbiResult};
use windy_riscv::{
    prelude::*,
    registers::{
        marchid, mimpid, mvendorid,
        sip::{sip, SIP},
    },
};

/// The major version of the SBI specification that is implemented
/// by this library.
pub const SBI_SPEC_MAJOR: usize = 0;
/// The minor version of the SBI specification that is implemented
/// by this library.
pub const SBI_SPEC_MINOR: usize = 2;

/// The raw bit representation of the SBI spec version that is implemented
/// bu this library.
///
/// The current implemented version is `v0.2`.
pub const SBI_SPEC_VERSION: usize = (SBI_SPEC_MAJOR << 24) | SBI_SPEC_MINOR;

// FIXME: Contribute to `riscv-sbi-doc` and add this ID.
/// The identifier for this implementation of the OpenSBI spec.
pub const SBI_IMPLEMENTATION_ID: usize = 6;

// FIXME: encode from real cargo version
/// The version of this implementation.
pub const SBI_IMPLEMENTATION_VERSION: usize = 1;

/// The unique identifier for the base extension.
pub const BASE_EXTENSION_ID: u32 = 0x10;
pub const TIMER_EXTENSION_ID: u32 = 0x54494D45;

/// A list of extensions that are supported by this
/// implementation.
pub const SUPPORTED_EXTENSIONS: &[u32] = &[BASE_EXTENSION_ID, TIMER_EXTENSION_ID];

/// Handles a SBI call that was caused by an `ecall` interrupt.
pub(crate) fn handle_ecall(eid: u32, fid: u32, args: [usize; 4]) -> SbiResult<usize> {
    if !SUPPORTED_EXTENSIONS.contains(&eid) {
        return Err(Error::NotSupported);
    }

    match eid {
        BASE_EXTENSION_ID => handle_base_ecall(fid, args[0]),
        TIMER_EXTENSION_ID => handle_timer_ecall(fid, args[0] as u64).map(|_| 0),
        _ => Err(Error::NotSupported),
    }
}

fn handle_base_ecall(fid: u32, arg: usize) -> SbiResult<usize> {
    match fid {
        // Return the current SBI spec version.
        0x00 => Ok(SBI_SPEC_VERSION),
        // Return the implementation id.
        0x01 => Ok(SBI_IMPLEMENTATION_ID),
        // Return the version of this implementation.
        0x02 => Ok(SBI_IMPLEMENTATION_VERSION),
        // Check if the given extension id is supported.
        0x03 => {
            if SUPPORTED_EXTENSIONS.contains(&(arg as u32)) {
                Ok(1)
            } else {
                Ok(0)
            }
        }
        // Return the `mvendorid` register
        0x04 => Ok(mvendorid::read()),
        // Return the `marchid` register
        0x05 => Ok(marchid::read()),
        // Return the `mimpi` register
        0x06 => Ok(mimpid::read()),
        _ => Err(Error::NotSupported),
    }
}

fn handle_timer_ecall(fid: u32, stime_value: u64) -> SbiResult<()> {
    // The timer extension only has a single function with id `0x00`.
    if fid != 0x00 {
        return Err(Error::NotSupported);
    }

    let platform = crate::platform::global();
    match &*platform.lock() {
        Some(platform) => {
            // Program the clock for the next event.
            (platform.set_timer)(stime_value);
            // clear the timer interrupt pending bit.
            sip().modify(SIP::STIP::CLEAR);

            Ok(())
        }
        // there's no global platform set, so return `NotSupported`.
        None => Err(Error::NotSupported),
    }
}
