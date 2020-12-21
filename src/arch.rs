#[cfg(feature = "virt")]
mod virt;
#[cfg(feature = "virt")]
pub use virt::*;

/// Returns the hard id where this function is called on.
#[inline(always)]
pub fn hart_id() -> usize {
    let mut hart_id: usize;
    unsafe { asm!("csrr {}, mhartid", out(reg) hart_id, options(nostack)) }
    hart_id
}

/// Loop forever
pub fn wait_forever() -> ! {
    loop {
        unsafe { asm!("wfi") }
    }
}
