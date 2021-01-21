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
    /// The privilige mode a trap in S-Mode was taken from.
    r SPP: 8..8 = SPP [
        /// The trap was taken from User mode.
        User = 0,
        /// The trap was taken from Supervisor mode.
        Supervisor = 1,
    ],
    /// The privilige mode a trap in M-Mode was taken from.
    r MPP: 11..12 = MPP [
        /// The trap was taken from User mode.
        User = 0b00,
        /// The trap was taken from Superivsor mode.
        Supervisor = 0b01,
        /// The trap was taken from Machine mode.
        Machine = 0b11,
    ],
    /// Encodes the state of the floating point unit.
    rw FS: 13..14 = FS [
        /// The FPU is disabled, aka. the floating point extension is not available
        /// on this system.
        Off = 0b00,
        /// The FPU is in the initialized state.
        Initial = 0b01,
        /// The FPU is fully cleaned up.
        Clean = 0b10,
        /// The FPU is dirty.
        Dirty = 0b11,
    ],

    /// Encodes the status of additional user-mode extensions and associated state.
    r XS: 15..16 = XS [
        AllOff = 0b00,
        NoneDirtyOrClean = 0b01,
        NoneDirtySomeClean = 0b10,
        SomeDirty = 0b11,
    ],

    /// Modifies the privilege level at which loads and stores are executed.
    ///
    /// If 0, loads and stores behave normal, using the translation process for the current
    /// privilege leve.
    /// If 1, loads and stores are translated and protected as if they are executed in the
    /// privilege level that is encoded in [`MPP`](mpp).
    rw MPRV: 17,

    /// Modifies the privilege with which S-Mode loads and stores access virtual memory.
    ///
    /// If 0, S-Mode memory access to pages that are accesible by U-Mode, will fault.
    /// If 1, those acceses are permitted.
    rw SUM: 18,

    /// Modifies the privilege level at which loads access virtual memory.
    ///
    /// If 0, only loads from pages that are marked as readable will succeed.
    /// If 1, loads from pages that are marked either readable or executable will
    /// succeed.
    rw MXR: 19,

    /// Supports intercepting virtual memery operations that are done
    /// in Supervisor mode.
    ///
    /// If 1, attempts to write to the `satp` register or executing the `SFENCE.VMA` instruction
    /// will raise an exception.
    /// If 0, those operations are permitted.
    rw TVM: 20,

    /// Supports intercepting the `WFI` instruction.
    ///
    /// If 1, and the `WFI` instruciton is executed in any less-privileged mode,
    /// and it doesn't complete in an implementation-specific timeout, an exception will be raised.
    /// If 0, the `WFI` instructions may freely be used by any less-privileged mode.
    rw TW: 21,

    /// Supports intercepting the `SRET` instruction.
    ///
    /// If 1, any attempt to execute the `SRET` instruction in S-Mode will raise an execption.
    /// If 0, `SRET` may freely be used in S-Mode.
    rw TSR: 22,
}
