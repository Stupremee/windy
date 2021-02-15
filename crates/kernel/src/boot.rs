use crate::{console, pmem};
use devicetree::DeviceTree;

/// Function that is run before `kinit` which is meant to setup paging and stuff
/// and then jumps into `kinit`.
#[no_mangle]
unsafe extern "C" fn _before_main(hart: usize, fdt: *const u8) -> ! {
    let tree = DeviceTree::from_ptr(fdt);
    let tree = tree.expect("failed to initialize devicetree");

    if console::init(&tree) {
        info!("{} Uart console", "Initialized".green());
    }

    pmem::init(&tree).expect("failed to initialize the physical memory allocator");

    crate::kinit(hart, &tree)
}

/// The entrypoint for the whole kernel.
///
/// `a0` = hart id
/// `a1` = pointer to device tree
#[naked]
#[no_mangle]
#[link_section = ".text.init"]
pub unsafe extern "C" fn _boot() -> ! {
    asm!(
        // ---------------------------------
        // Load the global pointer into
        // the `gp` register
        // ---------------------------------
        ".option push",
        ".option norelax",
        "    la gp, __global_pointer",
        ".option pop",
        // ---------------------------------
        // Disable interrupts
        // ---------------------------------
        "csrw sie, zero",
        "csrci sstatus, 2",
        // ---------------------------------
        // Set `bss` to zero
        // ---------------------------------
        "    la t0, __bss_start",
        "    la t1, __bss_end",
        "    bgeu t0, t1, zero_bss_done",
        "zero_bss:",
        "    sd zero, (t0)",
        "    addi t0, t0, 8",
        "zero_bss_done:",
        // ---------------------------------
        // Initialize stack
        // ---------------------------------
        "    la sp, __stack_end",
        // ---------------------------------
        // Jump into rust code
        // ---------------------------------
        "j _before_main",
        options(noreturn)
    )
}
