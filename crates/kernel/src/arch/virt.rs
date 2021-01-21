//! Architecture specific functions for Qemu and other emulators

const VIRT_TEST: *mut u32 = 0x10_0000 as *mut u32;

/// Shuts down the whole CPU (Emulator).
pub fn exit(code: u16) -> ! {
    let status = match code as u32 {
        0 => 0x5555,
        code => (code << 16) | 0x3333u32,
    };

    unsafe {
        core::ptr::write_volatile(VIRT_TEST, status);
    }

    unreachable!()
}
