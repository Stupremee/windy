//! Structure for representing traps and exceptions.

/// The bit that is set to `1`, inside the `cause` value, if a trap is
/// an interrupt.
pub const INTERRUPT_BIT: usize = 1 << (usize::BITS - 1);

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
pub enum Trap {
    UserSoftwareInterrupt,
    SupervisorSoftwareInterrupt,
    MachineSoftwareInterrupt,
    UserTimerInterrupt,
    SupervisorTimerInterrupt,
    MachineTimerInterrupt,
    UserExternalInterrupt,
    SupervisorExternalInterrupt,
    MachineExternalInterrupt,

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
    MachineModeEnvironmentCall,
    InstructionPageFault,
    LoadPageFault,
    StorePageFault,

    /// Special value that indicates an invalid cause,
    /// that may be valid in the future.
    Reserved,
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
