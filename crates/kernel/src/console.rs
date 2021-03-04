//! Implementation for accessing stdout/stdin.
//!
//! This module also contains print macros.

use crate::drivers;
use core::fmt::{self, Write};
use devicetree::{node::ChosenNode, DeviceTree};
use riscv::sync::{Mutex, MutexGuard};

pub static CONSOLE: Mutex<StaticConsoleDevice> = Mutex::new(StaticConsoleDevice(None));

/// Console device that can be used inside a static context.
pub struct StaticConsoleDevice(Option<ConsoleDevice>);

impl StaticConsoleDevice {
    /// Write the given byte into this device.
    ///
    /// If it hasn't initialized yet, it will be a no-op.
    pub fn write(&mut self, s: &str) -> fmt::Result {
        if let Some(ref mut dev) = self.0 {
            match dev {
                ConsoleDevice::NS16550(dev) => dev.write_str(s),
            }
        } else {
            Ok(())
        }
    }
}

impl fmt::Write for StaticConsoleDevice {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s)
    }
}

unsafe impl Send for StaticConsoleDevice {}
unsafe impl Sync for StaticConsoleDevice {}

/// All different kinds of devices that can be used as a console.
pub enum ConsoleDevice {
    NS16550(drivers::ns16550a::Device),
}

impl ConsoleDevice {
    /// Retrieve a console device from the `/chosen` node that is
    /// inside the devicetree.
    ///
    /// # Safety
    ///
    /// The given `node` must come from the devicetree to be a vaild node.
    pub unsafe fn from_chosen(node: &ChosenNode<'_>) -> Option<(Self, usize)> {
        let stdout = node.stdout()?;
        let mut compatible = stdout.prop("compatible")?.as_strings();

        if compatible.any(|name| drivers::ns16550a::COMPATIBLE.contains(&name)) {
            let addr = stdout.regions().next()?.start();
            Some((Self::NS16550(drivers::ns16550a::Device::new(addr)), addr))
        } else {
            None
        }
    }

    fn init(&mut self) {
        match self {
            ConsoleDevice::NS16550(dev) => dev.init(),
        }
    }
}

/// Initializes the global console by finding the right device that should
/// be used according to the given `/chosen` node of the given tree.
///
/// Returns `Some` with the physical address of the uart driver, if there's a uart device
pub fn init(tree: &DeviceTree<'_>) -> Option<usize> {
    if let Some((mut dev, addr)) = unsafe { ConsoleDevice::from_chosen(&tree.chosen()) } {
        dev.init();
        CONSOLE.lock().0 = Some(dev);
        Some(addr)
    } else {
        None
    }
}

/// Lock the console and return a guard that can write to the console.
pub fn lock() -> MutexGuard<'static, StaticConsoleDevice> {
    CONSOLE.lock()
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments<'_>) {
    let _ = CONSOLE.lock().write_fmt(args);
}
