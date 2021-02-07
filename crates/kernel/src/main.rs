#![deny(rust_2018_idioms, broken_intra_doc_links)]
#![no_std]
#![no_main]
#![feature(
    asm,
    cfg_target_has_atomic,
    naked_functions,
    exclusive_range_pattern,
    panic_info_message,
    array_value_iter,
    const_in_array_repeat_expressions
)]

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Windy can only run on 64 bit systems");

#[cfg(not(target_has_atomic = "ptr"))]
compile_error!("Windy can only run on systems that have atomic support");

pub mod arch;
pub mod console;
pub mod drivers;
pub mod log;
pub mod mem;
pub mod unit;

mod boot;
mod panic;

use devicetree::DeviceTree;

#[no_mangle]
unsafe extern "C" fn kinit(_hart_id: usize, fdt: *const u8) -> ! {
    let tree = DeviceTree::from_ptr(fdt).unwrap();

    console::init(&tree);
    mem::init(&tree).unwrap();

    println!("booting on {}", _hart_id);

    arch::exit(0)
}
