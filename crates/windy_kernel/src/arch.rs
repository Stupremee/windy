//! Functions and abstractions related to the RISC-V architecture.

#[cfg(feature = "virt")]
mod virt;
#[cfg(feature = "virt")]
pub use virt::*;

/// Returns the hard id where this function is called on.
#[inline(always)]
pub fn hart_id() -> usize {
    let mut hart_id: usize;
    unsafe { asm!("csrr {}, mhartid", out(reg) hart_id) }
    hart_id
}

/// Loop forever
#[inline]
pub fn wait_forever() -> ! {
    loop {
        unsafe { asm!("wfi") }
    }
}
