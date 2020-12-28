//! Function to access the SBI IPI extension functionality.

use super::{Error, SbiResult};

/// The unique id of the IPI extension.
pub const EXTENSION_ID: u32 = 0x735049;

/// Send an inter-process interrupt to all harts defined by the mask.
pub fn send_ipi(hart_mask: usize, hart_mask_base: usize) -> SbiResult<()> {
    let err_code: usize;
    unsafe {
        asm!("ecall",
            in("a7") EXTENSION_ID,
            in("a6") 0x00,

            in("a1") hart_mask_base,
            inout("a0") hart_mask => err_code,
        );
    }
    Error::from_sbi_call((), err_code as isize)
}
