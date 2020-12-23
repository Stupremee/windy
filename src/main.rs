#![deny(rust_2018_idioms, broken_intra_doc_links)]
#![no_std]
#![no_main]
#![feature(
    asm,
    naked_functions,
    cfg_target_has_atomic,
    panic_info_message,
    allocator_api,
    const_mut_refs,
    global_asm
)]

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Windy can only run on 64 bit systems");

#[cfg(not(target_has_atomic = "ptr"))]
compile_error!("Windy can only run on systems that have atomic support");

pub mod arch;
pub mod csr;
pub mod mem;
pub mod print;
pub mod trap;
pub mod uart;
pub mod utils;

mod boot;

use core::panic::PanicInfo;
use csr::{mscratch, mtvec};

#[no_mangle]
unsafe extern "C" fn kinit() -> ! {
    if arch::hart_id() != 0 {
        // Wait this halt forever as we currently only work
        // on hart `0`
        arch::wait_forever()
    }

    // Init logging and print hello message
    print::init_logging();
    log::info!("Hello from hart {}", arch::hart_id());

    // initialize trap handler
    let trap_frame = trap::kernel_trap_frame();
    mscratch::write(trap_frame);
    mtvec::write(trap::trap_vector_ptr() as _);

    // Initializing the memory allocators
    let mut alloc = mem::buddy::BuddyAllocator::new();
    let (start, end) = mem::heap_range();
    alloc.add_heap(start, end);

    println!("{}", alloc.stats());
    for _ in 0..1_000 {
        let res = alloc.alloc(0).unwrap();
    }
    println!("{}", alloc.stats());

    // Exit successfully
    arch::exit(0)
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
