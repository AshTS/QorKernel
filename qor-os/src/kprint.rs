#![allow(dead_code)]

use qor_core::interfaces::bytes::GenericByteWriteInterface;

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

/// Public function for writing to the UART port
pub fn uart_writer(s: &str) {
    let _ = crate::drivers::UART_DRIVER.send_bytes(s.as_bytes());
}

/// Set the logger in `qor_core` to use the UART port for logging
pub fn assign_uart_logger() {
    qor_core::logging::set_writer(&(uart_writer as fn(&str)));
    info!("Logger initialized to use UART port");
}
