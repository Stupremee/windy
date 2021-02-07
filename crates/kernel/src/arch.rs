//! Functions and abstractions related to the RISC-V architecture.

#[cfg(feature = "virt")]
mod virt;
#[cfg(feature = "virt")]
pub use virt::*;

use core::time::Duration;

// Loop forever
#[inline]
pub fn wait_forever() -> ! {
    loop {
        riscv::asm::wfi();
    }
}

/// Return the uptime of this hart.
pub fn time() -> Duration {
    let time = riscv::asm::rdtime();
    Duration::from_nanos(time as u64 * 100)
}
