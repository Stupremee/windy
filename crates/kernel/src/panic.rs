//! Custom Rust panic handler

use crate::println;
use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(info: &PanicInfo<'_>) -> ! {
    let mut _guard = crate::console::lock();

    crate::error!(guard = _guard; "============");
    crate::error!(guard = _guard; "KERNEL PANIC");
    crate::error!(guard = _guard; "============");
    if let Some(p) = info.location() {
        crate::error!(guard = _guard;
            "line {}, file {}: {}",
            p.line(),
            p.file(),
            info.message().unwrap()
        );
    } else {
        crate::error!(guard = _guard; "no information available.");
    }
    crate::arch::exit(1)
}
