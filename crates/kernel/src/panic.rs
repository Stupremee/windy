//! Custom Rust panic handler

use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(info: &PanicInfo<'_>) -> ! {
    //use core::fmt::Write;

    //let mut uart = crate::drivers::ns16550::Uart::new(0x1000_0000 as *mut _);

    //write!(uart, "Aborting: ").unwrap();
    //if let Some(p) = info.location() {
    //writeln!(
    //uart,
    //"line {}, file {}: {}",
    //p.line(),
    //p.file(),
    //info.message().unwrap()
    //)
    //.unwrap();
    //} else {
    //writeln!(uart, "no information available.").unwrap();
    //}
    crate::arch::exit(1)
}
