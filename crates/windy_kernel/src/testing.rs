//! Implementation of the Unit Test Runner to make unit
//! tests available on the RISC-V machine.
//!
//! This module should be guarded behind the `test` config.

use crate::drivers::ns16550::Uart;
use core::fmt::Write;

pub(super) fn test_runner(tests: &[&dyn Fn()]) {
    // FIXME: Replace with global `print` macro
    let mut uart = Uart::new(0x1000_0000 as *mut _);

    write!(uart, "Running {} tests\n", tests.len()).unwrap();
    for test in tests {
        test();
    }
}
