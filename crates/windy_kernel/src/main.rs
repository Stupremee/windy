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

use core::{
    fmt::{self, Write},
    ptr::NonNull,
};
use windy_devicetree::{DeviceTree, Uart};

#[no_mangle]
unsafe extern "C" fn kinit(_hart_id: usize, fdt: *const u8) -> ! {
    let mut uart = Uart::new();
    uart.init();
    let tree = DeviceTree::from_ptr(fdt).unwrap();

    //for node in tree.nodes_at_level(1) {
    //write!(uart, "{:?} => {}\n", node.name(), node.level()).unwrap();
    //}

    let cpus = tree.node("/").unwrap();
    for node in cpus.props() {
        write!(uart, "n: {:?}\n", node.name()).unwrap();
    }

    arch::exit(0)
}
