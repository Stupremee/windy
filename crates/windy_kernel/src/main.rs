#![deny(rust_2018_idioms, broken_intra_doc_links)]
#![no_std]
#![no_main]
#![feature(
    asm,
    cfg_target_has_atomic,
    naked_functions,
    custom_test_frameworks,
    exclusive_range_pattern,
    const_in_array_repeat_expressions
)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(crate::testing::test_runner)]

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Windy can only run on 64 bit systems");

#[cfg(not(target_has_atomic = "ptr"))]
compile_error!("Windy can only run on systems that have atomic support");

pub mod arch;
pub mod drivers;
pub mod mem;
pub mod unit;

mod boot;
mod panic;
#[macro_use]
mod macros;
#[cfg(test)]
mod testing;

use core::fmt::Write;
use windy_devicetree::DeviceTree;

#[no_mangle]
unsafe extern "C" fn kinit(_hart_id: usize, fdt: *const u8) -> ! {
    let mut uart = drivers::ns16550::Uart::new(0x1000_0000 as *mut _);

    let tree = DeviceTree::from_ptr(fdt).unwrap();
    let root = tree.memory();
    for c in root.regions() {
        write!(uart, "{:x?}\n", c).unwrap();
    }

    #[cfg(test)]
    crate::test_main();

    arch::exit(0)
}
