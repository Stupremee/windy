#![deny(rust_2018_idioms, broken_intra_doc_links)]
#![no_std]
#![no_main]
#![feature(asm, naked_functions)]

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Windy can only run on 64 bit systems");

pub mod arch;

mod boot;

use core::panic::PanicInfo;

#[no_mangle]
unsafe extern "C" fn kinit() -> ! {
    abort()
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    //abort()
    loop {}
}

#[no_mangle]
extern "C" fn abort() -> ! {
    const VIRT_TEST: *mut u32 = 0x10_0000 as *mut u32;

    unsafe {
        core::ptr::write_volatile(VIRT_TEST, 0x3333);
    }

    unreachable!()
}
