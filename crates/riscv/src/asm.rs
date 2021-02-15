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

/// Execute a `sfence.vma` instruction for the given address and asid.
#[inline]
pub fn sfence(addr: impl Into<Option<usize>>, asid: impl Into<Option<u16>>) {
    unsafe {
        match (addr.into(), asid.into()) {
            (Some(addr), Some(asid)) => asm!("sfence.vma {}, {}", in(reg) addr, in(reg) asid),
            (Some(addr), None) => asm!("sfence.vma {}, x0", in(reg) addr),
            (None, Some(asid)) => asm!("sfence.vma x0, {}", in(reg) asid),
            (None, None) => asm!("sfence.vma x0, x0"),
        }
    }
}
