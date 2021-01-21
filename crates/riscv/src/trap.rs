//! Structure for representing traps and exceptions.

/// The bit that is set to `1`, inside the `cause` value, if a trap is
/// an interrupt.
pub const INTERRUPT_BIT: usize = 1 << 63;

/// The context that is passed to the trap handler.
/// It stores all registers that will be restored after
/// the trap handler returned.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct TrapFrame {
    /// All 32 xregs, excluding `x0` which is always `0`.
    pub xregs: [usize; 31],
}

/// All different kinds of traps.
#[derive(Debug, Copy, Clone)]
#[repr(usize)]
pub enum Trap {
    UserSoftwareInterrupt = INTERRUPT_BIT | 0x00,
    SupervisorSoftwareInterrupt = INTERRUPT_BIT | 0x01,
    MachineSoftwareInterrupt = INTERRUPT_BIT | 0x03,
    UserTimerInterrupt = INTERRUPT_BIT | 0x04,
    SupervisorTimerInterrupt = INTERRUPT_BIT | 0x05,
    MachineTimerInterrupt = INTERRUPT_BIT | 0x07,
    UserExternalInterrupt = INTERRUPT_BIT | 0x08,
    SupervisorExternalInterrupt = INTERRUPT_BIT | 0x09,
    MachineExternalInterrupt = INTERRUPT_BIT | 0x0B,

    InstructionAddressMisaligned = 0x00,
    InstructionAccessFault = 0x01,
    IllegalInstruction = 0x02,
    Breakpoint = 0x03,
    LoadAddressMisaligned = 0x04,
    LoadAccessFault = 0x05,
    StoreAddressMisaligned = 0x06,
    StoreAccessFault = 0x07,
    UserModeEnvironmentCall = 0x08,
    SupervisorModeEnvironmentCall = 0x09,
    MachineModeEnvironmentCall = 0x0B,
    InstructionPageFault = 0x0C,
    LoadPageFault = 0x0D,
    StorePageFault = 0x0F,

    /// Special value that indicates an invalid cause,
    /// that may be valid in the future.
    Reserved = usize::MAX,
}

impl Trap {
    /// Converts a raw cause number coming from the `scause` register,
    /// into a [`Trap`].
    pub fn from_cause(cause: usize) -> Option<Self> {
        use Trap::*;

        const NON_INTERRUPT_TABLE: [Trap; 16] = [
            InstructionAddressMisaligned,
            InstructionAccessFault,
            IllegalInstruction,
            Breakpoint,
            LoadAddressMisaligned,
            LoadAccessFault,
            StoreAddressMisaligned,
            StoreAccessFault,
            UserModeEnvironmentCall,
            SupervisorModeEnvironmentCall,
            Reserved,
            MachineModeEnvironmentCall,
            InstructionPageFault,
            LoadPageFault,
            Reserved,
            StorePageFault,
        ];

        const INTERRUPT_TABLE: [Trap; 12] = [
            UserSoftwareInterrupt,
            SupervisorSoftwareInterrupt,
            Reserved,
            MachineSoftwareInterrupt,
            UserTimerInterrupt,
            SupervisorTimerInterrupt,
            Reserved,
            MachineTimerInterrupt,
            UserExternalInterrupt,
            SupervisorExternalInterrupt,
            Reserved,
            MachineExternalInterrupt,
        ];

        if cause & INTERRUPT_BIT != 0 {
            let cause = cause & !INTERRUPT_BIT;
            INTERRUPT_TABLE.get(cause).copied()
        } else {
            NON_INTERRUPT_TABLE.get(cause).copied()
        }
    }
}

impl TrapFrame {
    pub fn sp(&self) -> usize {
        self.xregs[1]
    }

    pub fn sp_ref(&mut self) -> &mut usize {
        &mut self.xregs[1]
    }

    pub fn a0(&self) -> usize {
        self.xregs[9]
    }

    pub fn a0_ref(&mut self) -> &mut usize {
        &mut self.xregs[9]
    }

    pub fn a1(&self) -> usize {
        self.xregs[10]
    }

    pub fn a1_ref(&mut self) -> &mut usize {
        &mut self.xregs[10]
    }

    pub fn a2(&self) -> usize {
        self.xregs[11]
    }

    pub fn a2_ref(&mut self) -> &mut usize {
        &mut self.xregs[11]
    }

    pub fn a3(&self) -> usize {
        self.xregs[12]
    }

    pub fn a3_ref(&mut self) -> &mut usize {
        &mut self.xregs[12]
    }

    pub fn a4(&self) -> usize {
        self.xregs[13]
    }

    pub fn a4_ref(&mut self) -> &mut usize {
        &mut self.xregs[13]
    }

    pub fn a5(&self) -> usize {
        self.xregs[14]
    }

    pub fn a5_ref(&mut self) -> &mut usize {
        &mut self.xregs[14]
    }

    pub fn a6(&self) -> usize {
        self.xregs[15]
    }

    pub fn a6_ref(&mut self) -> &mut usize {
        &mut self.xregs[15]
    }

    pub fn a7(&self) -> usize {
        self.xregs[16]
    }

    pub fn a7_ref(&mut self) -> &mut usize {
        &mut self.xregs[16]
    }
}
