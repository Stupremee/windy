//! Custom Rust panic handler

use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(info: &PanicInfo<'_>) -> ! {
    let mut _guard = crate::console::lock();

    crate::error!(guard = _guard; "============");
    crate::error!(guard = _guard; "KERNEL PANIC");
    crate::error!(guard = _guard; "============");

    match (info.location(), info.message()) {
        (Some(loc), Some(msg)) => {
            crate::error!(guard = _guard; "line {}, file {}: {}", loc.line(), loc.file(), msg)
        }
        (None, Some(msg)) => {
            crate::error!(guard = _guard; "{}", msg)
        }
        (Some(loc), None) => {
            crate::error!(guard = _guard; "line {}, file {}", loc.line(), loc.file())
        }
        (None, None) => crate::error!(guard = _guard; "no information available."),
    }

    crate::arch::exit(1)
}
