//! Function to access the SBI System Reset extension functionality.

use super::{Error, SbiResult};

/// The unique id of the System Reset extension.
pub const EXTENSION_ID: u32 = 0x53525354;

/// Specifies the reset type in a `reset` SBI call.
#[derive(Debug, Clone, Copy)]
pub enum Type {
    Shutdown,
    ColdReboot,
    WarmReboot,
    /// Raw number that will be passed to the type argument.
    Custom(usize),
}

/// The reason a system reset is happening.
#[derive(Debug, Clone, Copy)]
pub enum Reason {
    NoReason,
    SystemFailure,
    /// Raw number that will be passed to the reason argument.
    Custom(usize),
}

/// Reset the system based on the provided arguments.
pub fn reset(type_: Type, reason: Reason) -> SbiResult<!> {
    let err_code: usize;

    let type_ = match type_ {
        Type::Shutdown => 0x00,
        Type::ColdReboot => 0x01,
        Type::WarmReboot => 0x02,
        Type::Custom(val) => val,
    };

    let reason = match reason {
        Reason::NoReason => 0x00,
        Reason::SystemFailure => 0x01,
        Reason::Custom(val) => val,
    };

    unsafe {
        asm!("ecall",
            inout("a7") EXTENSION_ID => _,
            inout("a6") 0x00 => _,

            inout("a0") type_ => err_code,
            inout("a1") reason => _,
        );
    }

    match err_code {
        0 => unreachable!("`system_reset` sbi call should never return if no error occurred."),
        err => Err(Error::from_code(err as isize)),
    }
}

/// Shuts down the system with no reason.
pub fn shutdown() -> SbiResult<!> {
    reset(Type::Shutdown, Reason::NoReason)
}
