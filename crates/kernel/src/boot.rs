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
        // Initialize 64KiB stack
        // for **every** hart
        //
        // # Stack Layout:
        //
        // +-- _bss_end                    _bss_end + 0x80000 --+
        // |                                                    |
        // v                      512KiB                        v
        // +----------------------------------------------------+
        // | Hart 1 | Hart 2 | Hart 3 | Hart 4 |  unused stack  |
        // | 64 KiB | 64 KiB | 64 KiB | 64 KiB |     space      |
        // +----------------------------------------------------+
        //          ^        ^        ^        ^
        //        sp for   sp for    ...      ...
        //        hart 1   hart 2
        //
        // ---------------------------------
        "    la sp, __stack_start",
        // Load the stack size into `t0`
        "    li t0, 0x10000",
        // Load the hart id into `t1`
        "    mv t1, a0",
        // Increment it by one because hart ids start with zero.
        "    addi t1, t1, 1",
        // Multiply the stack size with the hart id to get
        // the offset for this hart inside the global stack
        "    mul t0, t0, t1",
        // Add the offset to the stack pointer
        "    add sp, sp, t0",
        // ---------------------------------
        // Jump into rust code
        // ---------------------------------
        "call kinit",
        options(noreturn)
    )
}
