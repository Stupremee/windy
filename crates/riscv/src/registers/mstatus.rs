//! Abstractions for the `mstatus` CSR.

read_csr!(pub 0x300);
write_csr!(pub 0x300);

csr_bits! { 0x300,
    /// Enable interrupts in user mode.
    rw UIE: 0,
    /// Enable interrupts in supervisor mode.
    rw SIE: 1,
    /// Enable interrupts in machine mode.
    rw MIE: 3,
    /// Holds the value of the interrupt-enable bit active prior to the trap.
    r UPIE: 4,
    /// Holds the value of the interrupt-enable bit active prior to the trap.
    r SPIE: 5,
    /// Holds the value of the interrupt-enable bit active prior to the trap.
    r MPIE: 7,
}
