//! Function to access the SBI base extension functionality.

use super::{Error, SbiResult};

/// The unique id of the base extension.
pub const EXTENSION_ID: u32 = 0x10;

fn sbi_call(fid: u32) -> SbiResult<usize> {
    let (value, err_code): (usize, isize);
    unsafe {
        asm!("ecall",
            in("a7") EXTENSION_ID,
            in("a6") fid,
            out("a0") err_code,
            out("a1") value
        );
    }

    Error::from_sbi_call(value, err_code)
}

/// Makes an `ecall` that will get the version
/// of the SBI spec that is used.
///
/// Returns the `(major, minor)` pair to indicate the version.
pub fn spec_version() -> SbiResult<(usize, usize)> {
    sbi_call(0x00).map(|raw_version| {
        let minor = raw_version & 0xFF_FFFF;
        let major = (raw_version >> 24) & 0x7F;
        (major, minor)
    })
}

/// Returns the current SBI implementation ID.
pub fn impl_id() -> SbiResult<usize> {
    sbi_call(0x01)
}

/// Returns the current SBI implementation version.
pub fn impl_version() -> SbiResult<usize> {
    sbi_call(0x02)
}

/// Checks if the given extension id is available.
pub fn probe_ext(ext: u32) -> SbiResult<bool> {
    let (value, err_code): (usize, usize);
    unsafe {
        asm!("ecall",
            in("a7") EXTENSION_ID,
            in("a6") 0x03,
            inout("a0") ext as usize => err_code,
            out("a1") value,
        );
    }

    Error::from_sbi_call(value, err_code as isize).map(|result| result != 0)
}

/// Returns the value of the `mvendorid` CSR.
pub fn mvendorid() -> SbiResult<usize> {
    sbi_call(0x04)
}

/// Returns the value of the `marchid` CSR.
pub fn marchid() -> SbiResult<usize> {
    sbi_call(0x05)
}

/// Returns the value of the `mimpid` CSR.
pub fn mimpid() -> SbiResult<usize> {
    sbi_call(0x06)
}
