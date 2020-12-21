read_csr!(Mtvec, 0x305);
write_csr!(0x305);

impl Mtvec {
    /// Returns the raw bits of this `mtvec` value.
    pub fn bits(&self) -> usize {
        self.bits
    }

    /// Returns the trap-vector base address.
    pub fn address(&self) -> usize {
        self.bits >> 2
    }

    /// Returns the trap-vector mode.
    pub fn mode(&self) -> Option<TrapMode> {
        let mode = self.bits & 0b11;
        match mode {
            0 => Some(TrapMode::Direct),
            1 => Some(TrapMode::Vectored),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TrapMode {
    /// All exceptions set `pc` to `BASE`.
    Direct = 0,
    /// Asynchronous interrupts set `pc` to `BASE + 4 * cause`.
    Vectored = 1,
}

/// Write the base address and the given mode into the `mtvec` register.
#[inline]
pub fn write(base: usize, mode: TrapMode) {
    let bits = (base << 2) | mode as usize;
    unsafe {
        _write(bits);
    }
}
