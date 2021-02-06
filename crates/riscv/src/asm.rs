//! Safe wrappers around some assembly instructions.

/// Read the cycle counter.
pub fn rdcycle() -> usize {
    let x: usize;
    unsafe { asm!("rdcycle {}", out(reg) x) };
    x
}

/// Read the instructions-retired counter.
pub fn rdinstret() -> usize {
    let x: usize;
    unsafe { asm!("rdinstret {}", out(reg) x) };
    x
}

/// Read the real time clock.
pub fn rdtime() -> usize {
    let x: usize;
    unsafe { asm!("rdtime {}", out(reg) x) };
    x
}
