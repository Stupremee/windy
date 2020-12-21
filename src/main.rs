#![deny(rust_2018_idioms, broken_intra_doc_links)]
#![no_std]
#![no_main]
#![feature(asm, naked_functions, cfg_target_has_atomic, panic_info_message)]

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Windy can only run on 64 bit systems");

#[cfg(not(target_has_atomic = "ptr"))]
compile_error!("Windy can only run on systems that have atomic support");

pub mod arch;
pub mod csr;
pub mod print;
pub mod trap;
pub mod uart;

mod boot;

use core::panic::PanicInfo;
use csr::mtvec;

#[no_mangle]
unsafe extern "C" fn kinit() -> ! {
    // Init logging and print hello message
    print::init_logging();
    log::info!("Hello from hart {}", arch::hart_id());

    // Set the trap handler
    mtvec::write(trap::trap_vector as usize, mtvec::TrapMode::Direct);

    // Wait this halt forever
    arch::wait_forever()
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    use core::fmt::Write;

    let mut guard = uart::uart().lock();

    // we can safely unwrap here because the `Write` implementation
    // for panic will never fail.
    write!(guard, "Aborting: ").unwrap();
    if let Some(p) = info.location() {
        writeln!(
            guard,
            "line {}, file {}: {}",
            p.line(),
            p.file(),
            info.message().unwrap()
        )
        .unwrap();
    } else {
        writeln!(guard, "no information available.").unwrap();
    }
    arch::exit(1)
}
