use qor_core::structures::id::HartID;

use crate::memory::Page;

#[repr(C)]
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrapFrame {
    pub registers: [u64; 32],
    pub floating_point_registers: [u64; 32],
    pub satp: u64,
    pub trap_stack: *mut Page,
    pub trap_stack_size: usize,
    pub hart_id: HartID,
}

/// Set the active trap frame for this hart.
///
/// # Safety
///
/// The pointer passed must be a properly aligned and valid pointer to a `TrapFrame` which will live at least until the next context switch.
#[allow(clippy::module_name_repetitions)]
pub fn set_trap_frame(ptr: &'static mut TrapFrame) {
    riscv::register::mscratch::write(ptr as *mut TrapFrame as usize);
}

unsafe impl core::marker::Sync for TrapFrame {}
