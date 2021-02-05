#![deny(rust_2018_idioms, broken_intra_doc_links)]
#![no_std]
#![no_main]
#![feature(
    asm,
    cfg_target_has_atomic,
    naked_functions,
    custom_test_frameworks,
    exclusive_range_pattern,
    panic_info_message
)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(crate::testing::test_runner)]

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Windy can only run on 64 bit systems");

#[cfg(not(target_has_atomic = "ptr"))]
compile_error!("Windy can only run on systems that have atomic support");

pub mod arch;
pub mod drivers;
pub mod unit;

mod boot;
mod panic;
#[cfg(test)]
mod testing;

#[no_mangle]
unsafe extern "C" fn kinit(_hart_id: usize, _fdt: *const u8) -> ! {
    //alloc.init(start, end).unwrap();
    //write!(uart, "{}\n", alloc.stats()).unwrap();

    #[cfg(test)]
    crate::test_main();

    arch::exit(0)
}
