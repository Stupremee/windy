#![deny(rust_2018_idioms, broken_intra_doc_links)]
#![no_std]
#![no_main]
#![feature(asm, cfg_target_has_atomic, naked_functions)]

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Windy can only run on 64 bit systems");

#[cfg(not(target_has_atomic = "ptr"))]
compile_error!("Windy can only run on systems that have atomic support");

pub mod arch;

mod boot;
mod panic;
#[macro_use]
mod macros;

#[no_mangle]
unsafe extern "C" fn kinit() -> ! {
    if arch::hart_id() != 0 {
        arch::wait_forever();
    }

    dbg!();
    sbi::init_sbi_handler();
    asm!("ecall");
    arch::exit(1);
}
