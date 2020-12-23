/// The bit that will indicate if the trap is an interrupt.
pub const INTERRUPT_BIT: usize = 1 << 63;

extern "C" {
    static mut trap_vector: u8;
}

/// The global trap frame for the kernel
static mut KERNEL_TRAP_FRAME: [TrapFrame; 1] = [TrapFrame::zero()];

/// Returns the pointer to the global kernel trap frame
pub unsafe fn kernel_trap_frame() -> *mut TrapFrame {
    &mut KERNEL_TRAP_FRAME[0] as *mut _
}

/// Returns a pointer to the global trap vector.
pub unsafe fn trap_vector_ptr() -> extern "C" fn() {
    let ptr = &mut trap_vector;
    core::mem::transmute::<_, _>(ptr)
}

/// The context for that is passed to the trap handler.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct TrapFrame {
    /// All 32 xregs, including `x0` which is `0`.
    pub xregs: [usize; 32],
    /// The `satp` register
    pub satp: usize,
    /// The hart id of the hart this trap occurred in.
    pub hartid: usize,
}

impl TrapFrame {
    pub const fn zero() -> Self {
        TrapFrame {
            xregs: [0; 32],
            satp: 0,
            hartid: 0,
        }
    }
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

/// The actual trap handler that executes code when a trap occurrs.
#[no_mangle]
pub unsafe extern "C" fn trap_handler(
    frame: &mut TrapFrame,
    cause: usize,
    tval: usize,
    epc: usize,
) -> usize {
    let cause = Trap::from_cause(cause);

    log::debug!("Trap handler called");
    log::debug!("Trap cause: {:?}", cause);
    log::debug!("Trap tval: {:x}", tval);
    log::debug!("Trap frame: {:x?}", frame);

    crate::arch::exit(1)
}

// TODO: Maybe have to disable interrupts here so we dont
// end up calling this multiple times?
global_asm!(
    "
    .section .text
    .global trap_vector
    # Align here is important, because lower two bits
    # must be zero.
    .align 4
    trap_vector:
        # we atomically switch `t6` and `mscratch` because,
        # the global trap frame is stored inside the `mscratch`
        # emote, and we use `mscratch` to save the `t6` register.
        csrrw t6, mscratch, t6

        # now store all register states inside the trap frame
        sd x1, 8(t6)
        sd x2, 16(t6)
        sd x3, 24(t6)
        sd x4, 32(t6)
        sd x5, 40(t6)
        sd x6, 48(t6)
        sd x7, 56(t6)
        sd x8, 64(t6)
        sd x9, 72(t6)
        sd x10, 80(t6)
        sd x11, 88(t6)
        sd x12, 96(t6)
        sd x13, 104(t6)
        sd x14, 112(t6)
        sd x15, 120(t6)
        sd x16, 128(t6)
        sd x17, 136(t6)
        sd x18, 144(t6)
        sd x19, 152(t6)
        sd x20, 160(t6)
        sd x21, 168(t6)
        sd x22, 176(t6)
        sd x23, 184(t6)
        sd x24, 192(t6)
        sd x25, 200(t6)
        sd x26, 208(t6)
        sd x27, 216(t6)
        sd x28, 224(t6)
        sd x29, 232(t6)
        sd x30, 240(t6)

        # now save the actual `t6` register value,
        # by storing `trap_frame` inside `t5`
        mv t5, t6
        csrr t6, mscratch
        sd x31, 248(t5)

        # restore `mscratch` register, the original value
        # currently lives in `t5`
        csrw mscratch, t5

        # prepare to jump into rust
        mv a0, t5
        csrr a1, mcause
        csrr a2, mtval
        csrr a3, mepc

        # read stack pointer from the trap frame
        # ld sp, 24(a0)

        # jump into rust code
        call trap_handler

        # rust function will return the new epc value, let's restore it
        csrw mepc, a0

        # restore old trap frame
        csrr t6, mscratch

        # now rstore all register states from the trap frame
        ld x1, 8(t6)
        ld x2, 16(t6)
        ld x3, 24(t6)
        ld x4, 32(t6)
        ld x5, 40(t6)
        ld x6, 48(t6)
        ld x7, 56(t6)
        ld x8, 64(t6)
        ld x9, 72(t6)
        ld x10, 80(t6)
        ld x11, 88(t6)
        ld x12, 96(t6)
        ld x13, 104(t6)
        ld x14, 112(t6)
        ld x15, 120(t6)
        ld x16, 128(t6)
        ld x17, 136(t6)
        ld x18, 144(t6)
        ld x19, 152(t6)
        ld x20, 160(t6)
        ld x21, 168(t6)
        ld x22, 176(t6)
        ld x23, 184(t6)
        ld x24, 192(t6)
        ld x25, 200(t6)
        ld x26, 208(t6)
        ld x27, 216(t6)
        ld x28, 224(t6)
        ld x29, 232(t6)
        ld x30, 240(t6)
        ld x31, 248(t6)

        # return from the trap using `mret`
        mret
    "
);
