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
    array_value_iter,
    const_in_array_repeat_expressions,
    slice_ptr_get,
    default_alloc_error_handler
)]

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Windy can only run on 64 bit systems");
#[cfg(not(target_has_atomic = "ptr"))]
compile_error!("Windy can only run on systems that have atomic support");

extern crate alloc;

pub mod arch;
pub mod console;
pub mod drivers;
pub mod log;
pub mod mem;
pub mod unit;

mod boot;
mod panic;

use devicetree::DeviceTree;
use displaydoc_lite::displaydoc;

/// The entry point for the booting hart.
#[no_mangle]
unsafe extern "C" fn kinit(hart_id: usize, fdt: *const u8) -> ! {
    match windy_main(hart_id, fdt) {
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
fn windy_main(_hart_id: usize, fdt: *const u8) -> Result<(), FatalError> {
    let tree = unsafe { DeviceTree::from_ptr(fdt) };
    let tree = tree.ok_or(FatalError::InvalidDeviceTree)?;

    if console::init(&tree) {
        info!("{} Uart console", "Initialized".green());
    }

    unsafe {
        mem::init(&tree).map_err(FatalError::Memory)?;
    }

    info!("{}", mem::alloc::allocator().stats());
    let _v = alloc::vec![0u8; 64 * unit::MIB];
    info!("{}", mem::alloc::allocator().stats());
    drop(_v);
    info!("{}", mem::alloc::allocator().stats());

    Ok(())
}

displaydoc! {
    /// Any error that will cause the kernel to exit.
    pub enum FatalError {
        /// The received device tree was invalid.
        InvalidDeviceTree,
        /// {_0}
        Memory(mem::Error),
    }
}
