//! Implementation of the trap handler.

#[naked]
unsafe extern "C" fn trap_handler() {
    asm!("", options(noreturn))
}
