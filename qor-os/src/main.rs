// Enable the ability to require functions to be four byte aligned
#![feature(fn_align)]
// Enable manual alignment checks
#![feature(pointer_is_aligned)]
#![no_std]
#![no_main]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

#[macro_use]
extern crate qor_core;

mod asm;
mod drivers;
mod kprint;
mod memory;
mod panic;

/// Entry point for the boot sequence, no interrupts are enabled when this function is called, and we are in machine
/// mode, no paging is enabled.
///
/// # Panics
/// This function will panic if a resource which is essential to the boot process is unavailable. For example, no UART
/// port being available, or there being insufficient memory to initialize the page table.
#[no_mangle]
#[repr(align(4))]
pub extern "C" fn kinit() {
    drivers::initialize_uart_driver().expect("Unable to initialize UART device driver");

    // Initialize the system logger to use the UART port
    kprint::assign_uart_logger();

    // Initialize the global page grained bump allocator
    memory::initialize_page_bump_allocator().expect("Unable to initialize bump allocator");
}
