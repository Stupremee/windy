#![deny(rust_2018_idioms, broken_intra_doc_links)]
#![allow(clippy::missing_safety_doc)]
#![no_std]
#![no_main]
#![feature(
    asm,
    cfg_target_has_atomic,
    naked_functions,
    exclusive_range_pattern,
    panic_info_message,
    slice_ptr_get,
    slice_ptr_len,
    int_bits_const,
    array_map
)]

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Windy can only run on 64 bit systems");
#[cfg(not(target_has_atomic = "ptr"))]
compile_error!("Windy can only run on systems that have atomic support");

pub mod arch;
#[macro_use]
pub mod console;
pub mod drivers;
#[macro_use]
pub mod log;
pub mod page;
pub mod pmem;
pub mod unit;

mod boot;
mod panic;

use devicetree::DeviceTree;
use displaydoc_lite::displaydoc;

/// The entry point for the booting hart.
#[no_mangle]
fn kinit(hart_id: usize, tree: &DeviceTree<'_>) -> ! {
    match windy_main(hart_id, tree) {
        Ok(()) => arch::exit(0),
        Err(err) => {
            error!("Failed to initialize kernel: {}", err.red());
            error!(
                "{} error happened while starting the kernel, exiting...",
                "Fatal".red()
            );
            arch::exit(1)
        }
    }
}

/// The "safe" entry point for the kernel.
fn windy_main(_hart_id: usize, tree: &DeviceTree<'_>) -> Result<(), Error> {
    for node in tree.find_nodes("/virtio_mmio") {
        info!("Tree node: {}", node.name());
    }

    Ok(())
}

displaydoc! {
    /// Any error that will cause the kernel to exit.
    pub enum Error {
    }
}
