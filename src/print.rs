//! `print` macros that use the UART driver to print the data.

use core::fmt::Arguments;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: Arguments<'_>) {
    use core::fmt::Write;

    let mut guard = crate::uart::uart().lock();
    guard.write_fmt(args).unwrap()
}

struct Logger;

impl log::Log for Logger {
    #[allow(unused_variables)]
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        #[cfg(any(debug_assertions, feature = "logging"))]
        return true;
        #[cfg(all(not(debug_assertions), not(feature = "logging")))]
        return metadata.level() <= log::Level::Info;
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            let mod_path = record
                .module_path_static()
                .or_else(|| record.module_path())
                .unwrap_or("<n/a>");

            println!("[ {:>5} ] [{}] {}", record.level(), mod_path, record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init_logging() {
    log::set_logger(&Logger).expect("failed to init logging");
    log::set_max_level(log::LevelFilter::Trace);
}
