//! Safe wrappers around some assembly instructions.

/// Wrapper around the `wfi` instruction.
#[inline]
pub fn wfi() {
    unsafe { asm!("wfi") }
}

/// Read the cycle counter.
#[inline]
pub fn rdcycle() -> usize {
    let x: usize;
    unsafe { asm!("rdcycle {}", out(reg) x) };
    x
}

/// Read the instructions-retired counter.
#[inline]
pub fn rdinstret() -> usize {
    let x: usize;
    unsafe { asm!("rdinstret {}", out(reg) x) };
    x
}

/// Read the real time clock.
#[inline]
pub fn rdtime() -> usize {
    let x: usize;
    unsafe { asm!("rdtime {}", out(reg) x) };
    x
}
