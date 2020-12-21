//! Architecture specific functions for Qemu and other emulators

use core::num::NonZeroUsize;

const UART_BASE: usize = 0x1000_0000;

/// Returns the base address of the UART controller.
pub const fn uart_base() -> NonZeroUsize {
    // SAFETY
    // `UART_BASE` is not zero
    unsafe { NonZeroUsize::new_unchecked(UART_BASE) }
}

/// Exits the CPU.
pub fn exit(code: u16) -> ! {
    const VIRT_TEST: *mut u32 = 0x10_0000 as *mut u32;

    let status = match code as u32 {
        0 => 0x5555,
        code => (code << 16) | 0x3333u32,
    };

    unsafe {
        core::ptr::write_volatile(VIRT_TEST, status);
    }

    unreachable!()
}
