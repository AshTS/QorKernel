// Enable the ability to require functions to be four byte aligned
#![feature(fn_align)]
#![no_std]
#![no_main]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

#[macro_use]
extern crate qor_core;

mod asm;
mod drivers;
mod kprint;
mod panic;

/// Entry point for the boot sequence, no interrupts are enabled when this function is called, and we are in machine
/// mode, no paging is enabled.
#[no_mangle]
#[repr(align(4))]
pub extern "C" fn kinit() {
    drivers::initialize_uart_driver().expect("Unable to initialize UART device driver");

    // Initialize the system logger to use the UART port
    kprint::assign_uart_logger();
}
