#![allow(dead_code)]

// Flag to set if the output should be colored
pub const COLORED: bool = cfg!(feature="color") && true;

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



/// Print text from the kernel to the UART port
#[macro_export]
macro_rules! kprint
{
    ($($args:tt)+) => ({
        use core::fmt::Write;
		let _ = write!(&$crate::drivers::UART_DRIVER, $($args)+);
    });
}

/// Print text from the kernel to the UART port, and appends a new line to the end
#[macro_export]
macro_rules! kprintln
{
    () => ({kprint!("\r\n")});

    ($fmt:expr) => ({
        $crate::kprint!(concat!($fmt, "\r\n"))
    });

    ($fmt:expr, $($args:tt)+) => ({
        $crate::kprint!(concat!($fmt, "\r\n"), $($args)+)
    });
}

/// Log a trace message in the kernel to the UART port
#[cfg(feature = "show_trace")]
#[macro_export]
macro_rules! trace {
    ($fmt: literal, $($args:tt)+) => {
        kprintln!(concat!("{}", $fmt), $crate::kprint::TRACE, $($args)+)
    };

    ($fmt: literal) => {
        kprintln!("{}{}", $crate::kprint::TRACE, $fmt)
    };
}
#[cfg(not(feature = "show_trace"))]
#[macro_export]
macro_rules! trace {
    ($($args:tt)+) => {};
}

/// Log a debug message in the kernel to the UART port
#[cfg(feature = "show_debug")]
#[macro_export]
macro_rules! debug {
    ($fmt: literal, $($args:tt)+) => {
        kprintln!(concat!("{}", $fmt), $crate::kprint::DEBUG, $($args)+)
    };

    ($fmt: literal) => {
        kprintln!("{}{}", $crate::kprint::DEBUG, $fmt)
    };
}
#[cfg(not(feature = "show_debug"))]
#[macro_export]
macro_rules! debug {
    ($($args:tt)+) => {};
}


/// Log an info message in the kernel to the UART port
#[cfg(feature = "show_info")]
#[macro_export]
macro_rules! info {
    ($fmt: literal, $($args:tt)+) => {
        kprintln!(concat!("{}", $fmt), $crate::kprint::INFO, $($args)+)
    };

    ($fmt: literal) => {
        kprintln!("{}{}", $crate::kprint::INFO, $fmt)
    };
}
#[cfg(not(feature = "show_info"))]
#[macro_export]
macro_rules! info {
    ($($args:tt)+) => {};
}

/// Log a warning message in the kernel to the UART port
#[cfg(feature = "show_warn")]
#[macro_export]
macro_rules! warn {
    ($fmt: literal, $($args:tt)+) => {
        kprintln!(concat!("{}", $fmt), $crate::kprint::WARN, $($args)+)
    };

    ($fmt: literal) => {
        kprintln!("{}{}", $crate::kprint::WARN, $fmt)
    };
}
#[cfg(not(feature = "show_warn"))]
#[macro_export]
macro_rules! warn {
    ($($args:tt)+) => {};
}

/// Log an error message in the kernel to the UART port
#[cfg(feature = "show_error")]
#[macro_export]
macro_rules! error {
    ($fmt: literal, $($args:tt)+) => {
        kprintln!(concat!("{}", $fmt), $crate::kprint::ERROR, $($args)+)
    };

    ($fmt: literal) => {
        kprintln!("{}{}", $crate::kprint::ERROR, $fmt)
    };
}
#[cfg(not(feature = "show_error"))]
#[macro_export]
macro_rules! error {
    ($($args:tt)+) => {};
}