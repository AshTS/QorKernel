/// Log levels used by the logger
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Trace = 0,
    Info = 1,
    Debug = 2,
    Warn = 3,
    Error = 4,
    Disabled = 5,
}

#[derive(Clone, Copy)]
pub struct LoggerWrapper {
    logger_inner: &'static fn(&str),
}

impl core::fmt::Write for LoggerWrapper {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        (self.logger_inner)(s);
        Ok(())
    }
}

const fn default_value(_: &str) {}

static LOGGER_OBJECT: atomic_ref::AtomicRef<'static, fn(&str)> =
    atomic_ref::AtomicRef::new(Some(&(default_value as fn(&str))));
static LOG_LEVEL: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);

/// Returns a logging wrapper for the configured logger.
pub fn get_writer() -> LoggerWrapper {
    LoggerWrapper {
        logger_inner: LOGGER_OBJECT
            .load(core::sync::atomic::Ordering::Acquire)
            .unwrap_or(&(default_value as fn(&str))),
    }
}

/// Returns true if the log level passed is to be displayed with the current settings.
pub fn check_log_level(level: LogLevel) -> bool {
    level as u8 >= LOG_LEVEL.load(core::sync::atomic::Ordering::Acquire)
}

/// Set the currently configured writing function for logging.
pub fn set_writer(writer: &'static fn(&str)) {
    LOGGER_OBJECT.store(Some(writer), core::sync::atomic::Ordering::Release);
}

/// Set the minimum displayed error level.
pub fn set_log_level(level: LogLevel) {
    LOG_LEVEL.store(level as u8, core::sync::atomic::Ordering::Release);
}

// Flag to set if the output should be colored
pub const COLORED: bool = true;

// Colors
pub const RED: &str = if COLORED { "\x1b[31m" } else { "" };
pub const GREEN: &str = if COLORED { "\x1b[32m" } else { "" };
pub const YELLOW: &str = if COLORED { "\x1b[33m" } else { "" };
pub const BLUE: &str = if COLORED { "\x1b[34m" } else { "" };
pub const CLEAR: &str = if COLORED { "\x1b[0m" } else { "" };

pub const TRACE: &str = "[TRACE] ";
pub const DEBUG: &str = const_format::formatcp!("[{}DEBUG{}] ", BLUE, CLEAR);
pub const INFO: &str = const_format::formatcp!("[{}INFO {}] ", GREEN, CLEAR);
pub const WARN: &str = const_format::formatcp!("[{}WARN {}] ", YELLOW, CLEAR);
pub const ERROR: &str = const_format::formatcp!("[{}ERROR{}] ", RED, CLEAR);

/// Log a trace message to the configured logger
#[macro_export]
macro_rules! trace {
    ($fmt: literal, $($args:tt)+) => {{
        use core::fmt::Write;
        if qor_core::logging::check_log_level(qor_core::logging::LogLevel::Trace) {
            let _ = write!(qor_core::logging::get_writer(), concat!("{}", $fmt, "\n"), $crate::logging::TRACE, $($args)+);
        }
    }};

    ($fmt: literal) => {{
        use core::fmt::Write;
        if qor_core::logging::check_log_level(qor_core::logging::LogLevel::Trace) {
            let _ = write!($crate::logging::get_writer(), "{}{}\n", $crate::logging::TRACE, $fmt);
        }
    }};
}

/// Log a debug message to the configured logger
#[macro_export]
macro_rules! debug {
    ($fmt: literal, $($args:tt)+) => {{
        use core::fmt::Write;
        if qor_core::logging::check_log_level(qor_core::logging::LogLevel::Debug) {
            let _ = write!(qor_core::logging::get_writer(), concat!("{}", $fmt, "\n"), $crate::logging::DEBUG, $($args)+);
        }
    }};

    ($fmt: literal) => {{
        use core::fmt::Write;
        if qor_core::logging::check_log_level(qor_core::logging::LogLevel::Debug) {
            let _ = write!($crate::logging::get_writer(), "{}{}\n", $crate::logging::DEBUG, $fmt);
        }
    }};
}

/// Log an info message to the configured logger
#[macro_export]
macro_rules! info {
    ($fmt: literal, $($args:tt)+) => {{
        use core::fmt::Write;
        if qor_core::logging::check_log_level(qor_core::logging::LogLevel::Info) {
            let _ = write!(qor_core::logging::get_writer(), concat!("{}", $fmt, "\n"), $crate::logging::INFO, $($args)+);
        }
    }};

    ($fmt: literal) => {{
        use core::fmt::Write;
        if qor_core::logging::check_log_level(qor_core::logging::LogLevel::Info) {
            let _ = write!($crate::logging::get_writer(), "{}{}\n", $crate::logging::INFO, $fmt);
        }
    }};
}

/// Log a warning message to the configured logger
#[macro_export]
macro_rules! warn {
    ($fmt: literal, $($args:tt)+) => {{
        use core::fmt::Write;
        if qor_core::logging::check_log_level(qor_core::logging::LogLevel::Warn) {
            let _ = write!(qor_core::logging::get_writer(), concat!("{}", $fmt, "\n"), $crate::logging::WARN, $($args)+);
        }
    }};

    ($fmt: literal) => {{
        use core::fmt::Write;
        if qor_core::logging::check_log_level(qor_core::logging::LogLevel::Warn) {
            let _ = write!($crate::logging::get_writer(), "{}{}\n", $crate::logging::WARN, $fmt);
        }
    }};
}

/// Log an error message to the configured logger
#[macro_export]
macro_rules! error {
    ($fmt: literal, $($args:tt)+) => {{
        use core::fmt::Write;
        if qor_core::logging::check_log_level(qor_core::logging::LogLevel::Error) {
            let _ = write!(qor_core::logging::get_writer(), concat!("{}", $fmt, "\n"), $crate::logging::ERROR, $($args)+);
        }
    }};

    ($fmt: literal) => {{
        use core::fmt::Write;
        if qor_core::logging::check_log_level(qor_core::logging::LogLevel::Error) {
            let _ = write!($crate::logging::get_writer(), "{}{}\n", $crate::logging::ERROR, $fmt);
        }
    }};
}
