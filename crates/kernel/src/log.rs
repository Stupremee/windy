//! Logging Framework for the Kernel.

use core::fmt;
use owo_colors::{colors, Color, OwoColorize};

/// Represents any level of a log message.
pub trait Level {
    type Color: Color;

    const NAME: &'static str;
}

/// The debug log level.
pub enum Debug {}
impl Level for Debug {
    type Color = colors::Magenta;
    const NAME: &'static str = "Debug";
}

/// The info log level.
pub enum Info {}
impl Level for Info {
    type Color = colors::Cyan;
    const NAME: &'static str = "Info";
}

/// The warn log level.
pub enum Warn {}
impl Level for Warn {
    type Color = colors::Yellow;
    const NAME: &'static str = "Warn";
}

/// The error log level.
pub enum Error {}
impl Level for Error {
    type Color = colors::Red;
    const NAME: &'static str = "Error";
}

/// Log a debug message.
#[macro_export]
macro_rules! debug {
    ($($args:tt)+) => {
        $crate::log!(Debug, $($args)+);
    }
}

/// Log an info message.
#[macro_export]
macro_rules! info {
    ($($args:tt)+) => {
        $crate::log!(Info, $($args)+);
    }
}

/// Log a warn message.
#[macro_export]
macro_rules! warn {
    ($($args:tt)+) => {
        $crate::log!(Warn, $($args)+);
    }
}

/// Log an error message.
#[macro_export]
macro_rules! error {
    ($($args:tt)+) => {
        $crate::log!(Error, $($args)+);
    }
}

/// The stadnard logging macro.
#[macro_export]
macro_rules! log {
    ($level:ident, $($args:tt)+) => {{
        #[allow(unused_imports)]
        use ::owo_colors::OwoColorize;
        $crate::log::_log::<$crate::log::$level>(::core::module_path!(), ::core::format_args!($($args)*));
    }};
}

#[doc(hidden)]
pub fn _log<L: Level>(module: &str, args: fmt::Arguments<'_>) {
    let time = crate::arch::time();
    let secs = time.as_secs();
    let millis = time.subsec_millis();

    crate::println!(
        "{} {:>5} {} > {}",
        format_args!("[{:>6}.{:<03}]", secs, millis).dimmed(),
        L::NAME.fg::<L::Color>(),
        module,
        args
    );
}
