//! The trap handler that will parse the arguments provided
//! to the SBI call and then forward to the specific extension.

use windy_riscv::trap::{Trap, TrapFrame};

extern "C" {
    static mut __sbi_trap_handler: u8;
}

/// Return the raw address of the trap handler that should be written
/// into the `mtvec` register.
pub(crate) unsafe fn trap_handler_address() -> usize {
    &__sbi_trap_handler as *const _ as usize
}

#[no_mangle]
unsafe extern "C" fn __rust_sbi_trap_handler(
    frame: &mut TrapFrame,
    cause: usize,
    epc: usize,
) -> usize {
    if let Some(Trap::SupervisorModeEnvironmentCall) = Trap::from_cause(cause) {
        let eid = frame.a7() as u32;
        let fid = frame.a6() as u32;

        let args = [frame.a0(), frame.a1(), frame.a2(), frame.a3()];
        let result = crate::interface::handle_ecall(eid, fid, args);

        match result {
            Ok(value) => {
                // set the error code to `0`, aka successful
                *frame.a0_ref() = 0;
                // store the value in `a1`
                *frame.a1_ref() = value;
            }
            Err(err) => {
                // store the error code in `a0`.
                *frame.a0_ref() = err.code() as usize;
                // set the value to `0`
                *frame.a1_ref() = 0;
            }
        };
    }

    // Skip the `ecall` instruction that caused this interrupt
    epc + 4
}

// The raw Assembly Written trap handler.
global_asm!(
    "
    .section .text
    .global sbi_trap_handler
    # The trap handler has to be aligned to `4` bytes because
    # the lower two bits must be 0, because they represent the 
    # mode inside the `mtvec` register.
    .align 4
    __sbi_trap_handler:
        # Store the original stack pointer
        csrw mscratch, sp

        # Load the trap stack
        la sp, __trap_stack

        # Allocate stack space for the trap frame.
        addi sp, sp, -248

        # Store `x1` register
        sd x1, 0(sp)

        # Store the original stack pointer inside the trap frame
        csrr x1, mscratch
        sd x1, 8(sp)

        # Store all other registers inside the trap frame
        sd x3, 16(sp)
        sd x4, 24(sp)
        sd x5, 32(sp)
        sd x6, 40(sp)
        sd x7, 48(sp)
        sd x8, 56(sp)
        sd x9, 64(sp)
        sd x10, 72(sp)
        sd x11, 80(sp)
        sd x12, 88(sp)
        sd x13, 96(sp)
        sd x14, 104(sp)
        sd x15, 112(sp)
        sd x16, 120(sp)
        sd x17, 128(sp)
        sd x18, 136(sp)
        sd x19, 144(sp)
        sd x20, 152(sp)
        sd x21, 160(sp)
        sd x22, 168(sp)
        sd x23, 176(sp)
        sd x24, 184(sp)
        sd x25, 192(sp)
        sd x26, 200(sp)
        sd x27, 208(sp)
        sd x28, 216(sp)
        sd x29, 224(sp)
        sd x30, 232(sp)
        sd x31, 240(sp)

        # Now setup arguments and jump to Rust code
        # 1. Argument: reference to the trap frame
        # 2. Argument: `mcause` register to identify Trap kind
        # 3. Argument: `mepc` register to skip the `ecall` instruction
        mv a0, sp
        csrr a1, mcause
        csrr a2, mepc

        # Jump to Rust function
        call __rust_sbi_trap_handler

        # Store the returned value inside the `mepc` register.
        csrw mepc, a0

        # Restore all other registers from the trap frame
        ld x1, 0(sp)
        ld x3, 16(sp)
        ld x4, 24(sp)
        ld x5, 32(sp)
        ld x6, 40(sp)
        ld x7, 48(sp)
        ld x8, 56(sp)
        ld x9, 64(sp)
        ld x10, 72(sp)
        ld x11, 80(sp)
        ld x12, 88(sp)
        ld x13, 96(sp)
        ld x14, 104(sp)
        ld x15, 112(sp)
        ld x16, 120(sp)
        ld x17, 128(sp)
        ld x18, 136(sp)
        ld x19, 144(sp)
        ld x20, 152(sp)
        ld x21, 160(sp)
        ld x22, 168(sp)
        ld x23, 176(sp)
        ld x24, 184(sp)
        ld x25, 192(sp)
        ld x26, 200(sp)
        ld x27, 208(sp)
        ld x28, 216(sp)
        ld x29, 224(sp)
        ld x30, 232(sp)
        ld x31, 240(sp)

        # Load original stack pointer from `mscratch` CSR
        csrr sp, mscratch

        # Return from the trap
        mret
"
);
