//! Function to access the SBI HSM (Hart State Management) extension functionality.

use super::{Error, SbiResult};

/// The unique id of the HSM extension.
pub const EXTENSION_ID: u32 = 0x48534D;

/// Informs the SBI implementation to asynchronously start the hart with `hart_id`.
///
/// The hart will start execution at the given `start_addr` and the raw value of `arg`
/// will be put into `a1`.
pub fn start(hart_id: usize, start_addr: usize, arg: usize) -> SbiResult<()> {
    let err_code: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") hart_id => err_code,
            inout("a1") start_addr => _,
            inout("a2") arg => _,
            inout("a6") 0 => _,
            inout("a7") EXTENSION_ID => _,
        );
    }
    Error::from_sbi_call((), err_code)
}

/// Stops the current hart.
///
/// This method must be called with Supervisor and User interrupts disabled.
pub fn stop() -> SbiResult<!> {
    let err_code: usize;
    unsafe {
        asm!("ecall",
            inout("a7") EXTENSION_ID => _,
            inout("a6") 0x01 => _,
            out("a0") err_code,
        );
    }

    match err_code {
        0 => unreachable!("`hart_stop` sbi call should never return if no error occurred."),
        err => Err(Error::from_code(err as isize)),
    }
}

/// Represents the status of a hart.
#[derive(Debug, Clone, Copy)]
pub enum Status {
    /// Hart has started and is currently running.
    Started,
    /// Hart is stopped.
    Stopped,
    /// Hart was requested to start, and waits until it can start.
    StartRequestPending,
    /// Hart was requested to stop.
    StopRequestPending,
    /// Unknown status code.
    Unknown(usize),
}

/// Returns the current status of the hart with id `hart_id`.
pub fn status(hart_id: usize) -> SbiResult<Status> {
    let (value, err_code): (usize, usize);
    unsafe {
        asm!("ecall",
            inout("a7") EXTENSION_ID => _,
            inout("a6") 0x02 => _,
            inout("a0") hart_id => err_code,
            out("a1") value,
        );
    }

    match err_code {
        0 => match value {
            0 => Ok(Status::Started),
            1 => Ok(Status::Stopped),
            2 => Ok(Status::StartRequestPending),
            3 => Ok(Status::StopRequestPending),
            status => Ok(Status::Unknown(status)),
        },
        err => Err(Error::from_code(err as isize)),
    }
}
