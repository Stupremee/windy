/// The naked assembly vector trap that will save the register
/// state, call the actual trap handler, and then
/// restore the state.
#[naked]
#[no_mangle]
pub unsafe extern "C" fn trap_vector() -> ! {
    asm!("mret", options(noreturn))
}
