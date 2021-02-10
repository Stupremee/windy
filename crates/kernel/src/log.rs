//! Logging Framework for the Kernel.

use core::{
    fmt::{self, Write},
    marker::PhantomData,
    time::Duration,
};
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
    (guard = $guard:expr; $($args:tt)+) => {
        $crate::log!(guard = $guard; Debug, $($args)+);
    };

    ($($args:tt)+) => {
        $crate::log!(Debug, $($args)+);
    };
}

/// Log an info message.
#[macro_export]
macro_rules! info {
    (guard = $guard:expr; $($args:tt)+) => {
        $crate::log!(guard = $guard; Info, $($args)+);
    };

    ($($args:tt)+) => {
        $crate::log!(Info, $($args)+);
    };
}

/// Log a warn message.
#[macro_export]
macro_rules! warn {
    (guard = $guard:expr; $($args:tt)+) => {
        $crate::log!(guard = $guard; Warn, $($args)+);
    };

    ($($args:tt)+) => {
        $crate::log!(Warn, $($args)+);
    };
}

/// Log an error message.
#[macro_export]
macro_rules! error {
    (guard = $guard:expr; $($args:tt)+) => {
        $crate::log!(guard = $guard; Error, $($args)+);
    };

    ($($args:tt)+) => {
        $crate::log!(Error, $($args)+);
    };
}

/// The stadnard logging macro.
#[macro_export]
macro_rules! log {
    ($level:ident, $($args:tt)+) => {{
        #[allow(unused_imports)]
        use ::owo_colors::OwoColorize;

        let mut _guard = $crate::console::lock();
        $crate::log!(guard = _guard; $level, $($args)*)
    }};

    (guard = $guard:expr; $level:ident, $($args:tt)+) => {{
        #[allow(unused_imports)]
        use ::owo_colors::OwoColorize;
        $crate::log::_log::<$crate::log::$level, _>(&mut *$guard, ::core::module_path!(), ::core::format_args!($($args)*));
    }};
}

/// Custom implementation of the `dbg` macro.
#[macro_export]
macro_rules! dbg {
    () => {
        $crate::debug!("[{}:{}]", ::core::file!(), ::core::line!());
    };

    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::debug!("[{}:{}] {} = {:#?}", ::core::file!(), ::core::line!(),
                    ::core::stringify!($val), &tmp);
                tmp
            }
        }
    };

    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

struct LogWriter<'fmt, L, G> {
    prefix: bool,
    time: Duration,
    module: &'fmt str,
    _guard: &'fmt mut G,
    _level: PhantomData<L>,
}

impl<L: Level, G: Write> LogWriter<'_, L, G> {
    fn print_prefix(&mut self) -> fmt::Result {
        let secs = self.time.as_secs();
        let millis = self.time.subsec_millis();
        write!(
            self._guard,
            "{} {:>5} {} > ",
            format_args!("[{:>3}.{:<03}]", secs, millis).dimmed(),
            L::NAME.fg::<L::Color>(),
            self.module,
        )
    }
}

impl<L: Level, G: Write> fmt::Write for LogWriter<'_, L, G> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.prefix {
            self.print_prefix()?;
            self.prefix = false;
        }

        if let Some(newline) = s.find('\n') {
            let (s, rest) = s.split_at(newline + 1);
            self._guard.write_str(s)?;

            if !rest.is_empty() {
                self.print_prefix()?;
                self._guard.write_str(rest)?;
            } else {
                self.prefix = true;
            }
        } else {
            self._guard.write_str(s)?;
        }

        Ok(())
    }
}

#[doc(hidden)]
pub fn _log<L: Level, G: Write>(_guard: &mut G, module: &str, args: fmt::Arguments<'_>) {
    let mut writer = LogWriter {
        time: crate::arch::time(),
        prefix: true,
        module,
        _guard,
        _level: PhantomData::<L>,
    };

    writeln!(writer, "{}", args).expect("failed to log message");
}
