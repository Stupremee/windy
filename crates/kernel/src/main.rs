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
    slice_ptr_len,
    int_bits_const
)]

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Windy can only run on 64 bit systems");
#[cfg(not(target_has_atomic = "ptr"))]
compile_error!("Windy can only run on systems that have atomic support");

#[cfg(target_arch = "riscv")]
compile_error!("foo");

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
        pmem::init(&tree).map_err(FatalError::Memory)?;
    }

    debug!("{}", pmem::alloc_stats());

    let start = arch::time();

    const COUNT: usize = 15_000;
    const PAGE_COUNT: usize = 2;

    for _ in 0..COUNT {
        let ptr = pmem::alloc_pages(PAGE_COUNT).unwrap();
        assert!(ptr.len() >= 4096 * PAGE_COUNT)
    }

    let end = arch::time();
    let time = end - start;

    let bytes = 4096 * PAGE_COUNT * COUNT;
    debug!(
        "allocated {} pages {} times ({}) in {:?}",
        PAGE_COUNT,
        COUNT,
        unit::bytes(bytes),
        time,
    );

    debug!("{}", pmem::alloc_stats());

    Ok(())
}

displaydoc! {
    /// Any error that will cause the kernel to exit.
    pub enum FatalError {
        /// The received device tree was invalid.
        InvalidDeviceTree,
        /// {_0}
        Memory(pmem::Error),
    }
}
