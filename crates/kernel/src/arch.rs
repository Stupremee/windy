//! Functions and abstractions related to the RISC-V architecture.

#[cfg(feature = "virt")]
mod virt;
#[cfg(feature = "virt")]
pub use virt::*;

/// Loop forever
#[inline]
pub fn wait_forever() -> ! {
    loop {
        unsafe { asm!("wfi") }
    }
}
