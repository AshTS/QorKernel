#![no_std]

#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
)]

#[cfg(feature = "std")]
extern crate std;

pub mod drivers;