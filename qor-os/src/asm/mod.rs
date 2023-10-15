//! Include assembly files

use core::arch::global_asm;

global_asm!(include_str!("boot.s"));
global_asm!(include_str!("mem.s"));
global_asm!(include_str!("trap.s"));

// Values defined in assembly which now need to be brought into rust
extern "C" {
    pub static HEAP_START: *mut qor_riscv::memory::Page;
    pub static HEAP_END: *mut qor_riscv::memory::Page;

    pub static TEXT_START: *mut qor_riscv::memory::Page;
    pub static TEXT_END: *mut qor_riscv::memory::Page;

    pub static DATA_START: *mut qor_riscv::memory::Page;
    pub static DATA_END: *mut qor_riscv::memory::Page;

    pub static RODATA_START: *mut qor_riscv::memory::Page;
    pub static RODATA_END: *mut qor_riscv::memory::Page;

    pub static BSS_START: *mut qor_riscv::memory::Page;
    pub static BSS_END: *mut qor_riscv::memory::Page;

    pub static KERNEL_STACK_START: *mut qor_riscv::memory::Page;
    pub static KERNEL_STACK_END: *mut qor_riscv::memory::Page;
}
