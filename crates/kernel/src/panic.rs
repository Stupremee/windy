//! Custom Rust panic handler

use crate::println;
use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(info: &PanicInfo<'_>) -> ! {
    println!("!!! PANIC !!!");
    if let Some(p) = info.location() {
        println!(
            "line {}, file {}: {}",
            p.line(),
            p.file(),
            info.message().unwrap()
        );
    } else {
        println!("no information available.");
    }
    crate::arch::exit(1)
}
