#![allow(dead_code)]
use qor_riscv::trap::frame::TrapFrame;

use self::structures::TrapInfo;

pub mod external;
pub mod handle;
pub use handle::*;
pub mod structures;

/// Raw trap handler
#[no_mangle]
#[allow(clippy::module_name_repetitions)]
#[repr(align(4))]
pub extern "C" fn m_trap(
    epc: usize,
    tval: usize,
    cause: usize,
    hart: usize,
    status: usize,
    frame: &'static TrapFrame,
    _satp: usize,
) -> usize {
    let trap_info = TrapInfo::from_raw(epc, tval, cause, hart, status, frame);
    crate::trap::handle_trap(&trap_info)
}

/// Initialize the trap frame
pub fn initialize_trap_frame() {
    let stack = crate::memory::PAGE_BUMP_ALLOCATOR
        .allocate(2)
        .expect("Unable to allocate space for trap stack");

    let frame = TrapFrame {
        registers: [0; 32],
        floating_point_registers: [0; 32],
        satp: 0,
        trap_stack: unsafe { stack.as_mut_ptr().add(2) },
        trap_stack_size: 2,
        hart_id: qor_core::structures::id::HartID(0),
    };

    let frame = crate::memory::bump::PAGE_BUMP_ALLOCATOR
        .allocate_object(frame)
        .expect("Unable to allocate space for trap frame");
    qor_riscv::trap::frame::set_trap_frame(frame);
}

/// Allocate a trap frame
pub fn allocate_trap_frame() -> TrapFrame {
    let stack = crate::memory::PAGE_BUMP_ALLOCATOR
        .allocate(2)
        .expect("Unable to allocate space for trap stack");

    TrapFrame {
        registers: [0; 32],
        floating_point_registers: [0; 32],
        satp: 0,
        trap_stack: unsafe { stack.as_mut_ptr().add(2) },
        trap_stack_size: 2,
        hart_id: qor_core::structures::id::HartID(0),
    }
}
