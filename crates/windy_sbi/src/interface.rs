//! Implementation of the SBI extensions and the actual SBI
//! method execution.

use super::{Error, Platform, SbiResult};
use windy_riscv::{
    prelude::*,
    registers::{
        marchid, mimpid, mvendorid,
        sip::{sip, SIP},
    },
    trap::{Trap, TrapFrame},
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
/// The unique identifier for the timer extension.
pub const TIMER_EXTENSION_ID: u32 = 0x54494D45;

/// A list of extensions that are supported by this
/// implementation.
pub const SUPPORTED_EXTENSIONS: &[u32] = &[BASE_EXTENSION_ID, TIMER_EXTENSION_ID];

/// Handles an `ecall` trap
///
/// To enable the use of the SBI, you have to call this function in your
/// trap handler in M-Mode. The returned value contains the new `mepc`,
/// which should be set to the returned value if it's `Some`, otherwise
/// the SBI call was invalid.
///
/// This function will write the `a0` and `a1` registers inside the trap frame to
/// return error/value, so don't modify it afterwards.
///
/// # Example
/// ```ignore
/// const PLATFORM: windy_sbi::Platform = ...;
///
/// #[no_mangle]
/// unsafe extern "C" fn __rust_sbi_trap_handler(
///     frame: &mut TrapFrame,
///     cause: usize,
///     epc: usize,
/// ) -> usize {
///     if let Some(new_epc) = windy_sbi::ecall(PLATFORM, frame, cause, epc) {
///         new_epc
///     } else {
///         // handle trap that was not caused by `ecall`.
///     }
/// }
/// ```
pub fn ecall(
    platform: &Platform,
    frame: &mut TrapFrame,
    cause: usize,
    epc: usize,
) -> Option<usize> {
    if let Some(Trap::SupervisorModeEnvironmentCall) | Some(Trap::MachineModeEnvironmentCall) =
        Trap::from_cause(cause)
    {
        let eid = frame.a7() as u32;
        let fid = frame.a6() as u32;

        let args = [frame.a0(), frame.a1(), frame.a2(), frame.a3()];
        let result = handle_raw_ecall(platform, eid, fid, args);

        match result {
            Ok(value) => {
                // set the error code to `0`, aka successful
                *frame.a0_ref() = 0;
                // store the value in `a1`
                *frame.a1_ref() = value;
            }
            Err(err) => {
                // store the error code in `a0`.
                *frame.a0_ref() = err.code() as usize;
                // set the value to `0`
                *frame.a1_ref() = 0;
            }
        };

        // Skip the `ecall` instruction that caused this interrupt
        return Some(epc + 4);
    }

    // We didn't hit an `ecall`, so we will not handle the trap
    None
}

/// Handles a SBI call that was caused by an `ecall` interrupt.
fn handle_raw_ecall(platform: &Platform, eid: u32, fid: u32, args: [usize; 4]) -> SbiResult<usize> {
    if !SUPPORTED_EXTENSIONS.contains(&eid) {
        return Err(Error::NotSupported);
    }

    match eid {
        BASE_EXTENSION_ID => handle_base_ecall(fid, args[0]),
        TIMER_EXTENSION_ID => handle_timer_ecall(platform, fid, args[0] as u64).map(|_| 0),
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

fn handle_timer_ecall(platform: &Platform, fid: u32, stime_value: u64) -> SbiResult<()> {
    // The timer extension only has a single function with id `0x00`.
    if fid != 0x00 {
        return Err(Error::NotSupported);
    }

    // Program the clock for the next event.
    (platform.set_timer)(stime_value);
    // clear the timer interrupt pending bit.
    sip().modify(SIP::STIP::CLEAR);

    Ok(())
}
