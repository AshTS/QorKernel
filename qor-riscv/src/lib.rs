#![no_std]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

#[cfg(feature = "std")]
extern crate std;

#[allow(unused_imports)]
#[macro_use]
extern crate qor_core;

pub mod drivers;
pub mod memory;
pub mod trap;
