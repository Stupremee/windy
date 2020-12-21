/// The entrypoint for the whole kernel.
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
        "    la gp, _global_pointer",
        ".option pop",
        // ---------------------------------
        // Disable paging
        // ---------------------------------
        "csrw satp, zero",
        // ---------------------------------
        // Set `bss` to zero
        // ---------------------------------
        "    la a0, _bss_start",
        "    la a1, _bss_end",
        "    bgeu a0, a1, zero_bss_done",
        "zero_bss:",
        "    sd zero, (a0)",
        "    addi a0, a0, 8",
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
        "    la sp, _stack_start",
        // Load the stack size into `a0`
        "    li a0, 0x10000",
        // Load the hardid into `a1`
        "    csrr a1, mhartid",
        // Increment it by one because hart ids start with zero.
        "    addi a1, a1, 1",
        // Multiply the stack size with the hart id to get
        // the offset for this hart inside the global stack
        "    mul a0, a0, a1",
        // Add the offset to the stack pointer
        "    add sp, sp, a0",
        // ---------------------------------
        // Jump into rust code
        // ---------------------------------
        "call kinit",
        options(noreturn)
    )
}
