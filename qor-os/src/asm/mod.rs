//! Include assembly files

use core::arch::global_asm;

global_asm!(include_str!("boot.s"));