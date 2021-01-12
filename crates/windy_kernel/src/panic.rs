//! Custom Rust panic handler

use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(info: &PanicInfo<'_>) -> ! {
    if let Some(loc) = info.location() {
        use core::fmt::Write;

        let mut uart = crate::drivers::ns16550::Uart::new(0x1000_0000 as *mut _);
        writeln!(uart, "panic: {}", loc).unwrap();
    }
    crate::arch::exit(1);
}
