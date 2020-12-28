//! Function to access the SBI timer extension functionality.

use super::{Error, SbiResult};

/// The unique id of the timer extension.
pub const EXTENSION_ID: u32 = 0x54494D45;

/// Programs the clock for the next event after `stime`.
pub fn set_timer(stime: u64) -> SbiResult<()> {
    let err_code: usize;
    unsafe {
        asm!("ecall",
            in("a7") EXTENSION_ID,
            in("a6") 0x00,
            inout("a0") stime as usize => err_code,
        );
    }
    Error::from_sbi_call((), err_code as isize)
}
