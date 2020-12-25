//! Custom Rust panic handler

use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(_info: &PanicInfo<'_>) -> ! {
    crate::arch::exit(1);
}
