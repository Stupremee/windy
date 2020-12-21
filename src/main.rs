#![deny(rust_2018_idioms, broken_intra_doc_links)]
#![no_std]
#![no_main]
#![feature(asm, naked_functions, cfg_target_has_atomic)]

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Windy can only run on 64 bit systems");

//#[cfg(not(target_has_atomic))]
//compile_error!("Windy requires atomics");

pub mod arch;
pub mod print;
pub mod uart;

mod boot;

use core::panic::PanicInfo;

#[no_mangle]
unsafe extern "C" fn kinit() -> ! {
    if arch::hart_id() != 0 {
        arch::wait_forever()
    }
    print::init_logging();

    log::info!("Hello from hart {}", arch::hart_id());
    arch::exit(1);
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    log::error!("{}", info);
    arch::exit(1);
}
