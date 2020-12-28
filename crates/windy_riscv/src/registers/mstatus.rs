//! The `mstatus` CSR.

use register::{cpu::RegisterReadWrite, register_bitfields};

read_csr!(0x300);
write_csr!(0x300);

// the `u64` here is safe, because we can only build on 64-bit systems
register_bitfields! { u64,
    /// The `mstatus` register keeps track of the current state of
    /// the executing hart.
    pub MSTATUS [
        /// Globally enables interrupts for U-Mode.
        UIE OFFSET(0) NUMBITS(1) [],
        /// Globally enables interrupts for S-Mode.
        SIE OFFSET(1) NUMBITS(1) [],
        /// Globally enables interrupts for M-Mode.
        MIE OFFSET(3) NUMBITS(1) [],

        /// Holds the value for the interrupt-enable bit prior to the trap for U-Mode.
        UPIE OFFSET(4) NUMBITS(1) [],
        /// Holds the value for the interrupt-enable bit prior to the trap for S-Mode.
        SPIE OFFSET(5) NUMBITS(1) [],
        /// Holds the value for the interrupt-enable bit prior to the trap for M-Mode.
        MPIE OFFSET(7) NUMBITS(1) [],

        /// Contains the previous privilege mode for S-Mode.
        ///
        /// Can only be `S` or `U` mode
        SPP OFFSET(8) NUMBITS(1) [
            /// Sets `SPP` to user mode
            U = 0,
            /// Sets `SPP` to supervisor mode
            S = 1
        ],
        /// Contains the previous privilege mode for M-Mode.
        MPP OFFSET(11) NUMBITS(2) [
            /// Sets `MPP` to user mode
            U = 0b00,
            /// Sets `MPP` to supervisor mode
            S = 0b01,
            /// Sets `MPP` to machine mode
            M = 0b11
        ],

        /// Tracks the current state of the floating point unit.
        ///
        /// **This field is read only.**
        FS OFFSET(13) NUMBITS(2) [
            OFF = 0b00,
            INITIAL = 0b01,
            CLEAN = 0b10,
            DIRTY = 0b11
        ],

        /// Tracks the current state of additional user mode extensions
        /// and associated state.
        ///
        /// **This field is read only.**
        XS OFFSET(15) NUMBITS(2) [
            OFF = 0b00,
            INITIAL = 0b01,
            CLEAN = 0b10,
            DIRTY = 0b11
        ],

        /// Modifies the privilege level at which loads and stores are executed.
        ///
        /// If `0`, the load and store mechanism uses the normal translation
        /// mechanismus of the current privilege leve.
        /// If `1`, load and stores are translated and protected as the current
        /// privilege mode were set to `MPP`.
        MPRV OFFSET(17) NUMBITS(1) [],

        /// Modifies the privilege with which S-Mode load and stores access virtual memory.
        ///
        /// If `0`, S-Mode memory access to pages that are accessible by U-Mode will fault.
        /// If `1`, S-Mode memory access to pages that are accessible by U-Mode will succeed.
        SUM OFFSET(18) NUMBITS(1) [],

        /// Modifies the privilege with which loads access virtual memory.
        ///
        /// If `0`, only loads from pages that are marked as readable will succeed.
        /// If `1`, only loads from pages that are marked as readable or executable will succeed.
        MXR OFFSET(19) NUMBITS(1) [],

        /// Trap virtual memory intercepts virtual memory operations.
        ///
        /// If `1`, attempts to read `satp` CSR or executing `sfence.vma` instruction in S-Mode
        /// will raise illegal instruction exception.
        /// If `0`, these operations are permitted in S-Mode.
        TVM OFFSET(20) NUMBITS(1) [],

        /// Timtout wait intercepts the `wfi` instruction.
        ///
        /// If `0`, the `wfi` instruction may execute in lower privilege modes.
        /// If `1`, and the `wfi` instruction is executed in lower privilege modes,
        /// and it doesn't complete in an implementation-specific time, illegal instruction
        /// exception is raised.
        TW OFFSET(21) NUMBITS(1) [],

        /// Trap SRET intercepts the supervisors `sret` instruction.
        ///
        /// If `1`, attempts to execute `sret` in S-Mode will raise illegal instruction exception.
        /// If `0`, the `sret` instruction is allowed.
        TSR OFFSET(22) NUMBITS(1) []
    ]
}

pub struct MstatusRegister;

impl RegisterReadWrite<u64, MSTATUS::Register> for MstatusRegister {
    #[inline]
    fn get(&self) -> u64 {
        unsafe { _read() as u64 }
    }

    #[inline]
    fn set(&self, value: u64) {
        unsafe { _write(value as usize) };
    }
}

static REGISTER: MstatusRegister = MstatusRegister;

/// Returns a reference to a struct that can be used to
/// modify, read, write the register using the [`register`] crate.
///
/// [`register`]: https://docs.rs/register
pub fn mstatus() -> &'static MstatusRegister {
    &REGISTER
}
