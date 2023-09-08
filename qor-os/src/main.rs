// Enable the ability to require functions to be four byte aligned
#![feature(fn_align)]

#![no_std]
#![no_main]

#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
)]

mod asm;
mod panic;
#[no_mangle]
#[repr(align(4))]
pub extern "C" fn kinit() {

}