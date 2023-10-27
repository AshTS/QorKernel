#![no_std]
// Allow doing pointer arithmetic
#![feature(ptr_sub_ptr)]
// Require nice atomic pointers
#![feature(strict_provenance_atomic_ptr)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod drivers;
pub mod interfaces;

#[macro_use]
pub mod logging;

pub mod memory;
pub mod structures;
#[cfg(feature = "alloc")]
pub mod tasks;
pub mod utils;
