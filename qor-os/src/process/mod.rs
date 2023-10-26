#![allow(dead_code)]

use core::sync::atomic::AtomicU16;

use qor_core::{memory::allocators::page::bitmap::PageBox, structures::id::PID};
use qor_riscv::{
    memory::{mmu::entry::GlobalUserFlags, Page, PageCount},
    trap::frame::TrapFrame,
};

use crate::{
    memory::{get_page_bitmap_allocator, mmu::ManagedPageTable, PageSequence},
    trap::allocate_trap_frame,
};

static PID_COUNTER: AtomicU16 = AtomicU16::new(1);

/// Get the next PID to be used
fn new_pid() -> PID {
    PID(PID_COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed))
}

/// Execution state for process execution. Includes a trap frame (which doesn't store the information for executing
/// traps, but for executing user mode), a program counter storing where in the executable we return to, and a sequence
/// of pages used for the stack.
pub struct ExecutionState {
    program_counter: usize,
    stack: PageSequence,
    trap_frame: PageBox<'static, Page, TrapFrame>,
}

#[allow(clippy::module_name_repetitions)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Active,
    Running,
    Sleeping,
    Waiting,
    Terminated,
}

#[repr(C)]
pub struct Process {
    pid: PID,
    main_execution: ExecutionState,
    state: ProcessState,
    page_table: PageBox<'static, Page, ManagedPageTable>,
}

impl ExecutionState {
    pub fn from_components(initial_program_counter: usize, stack_size: PageCount) -> Self {
        Self {
            program_counter: initial_program_counter,
            stack: PageSequence::alloc(stack_size.raw()),
            trap_frame: get_page_bitmap_allocator()
                .alloc_boxed(allocate_trap_frame())
                .expect("Unable to allocate trap frame space"),
        }
    }
}

extern "C" {
    pub fn switch_to_user(frame: usize, pc: usize, satp: usize) -> !;
}

impl Process {
    pub fn from_components(
        execution_state: ExecutionState,
        page_table: PageBox<'static, Page, ManagedPageTable>,
    ) -> Self {
        Self {
            pid: new_pid(),
            main_execution: execution_state,
            state: ProcessState::Active,
            page_table,
        }
    }

    pub fn from_fn_ptr(function: usize, stack_size: PageCount) -> Self {
        let mut page_table = crate::memory::get_page_bitmap_allocator()
            .alloc_boxed(crate::memory::mmu::ManagedPageTable::empty())
            .expect("Unable to allocate space for process page table");
        crate::memory::mmu::identity_map_kernel(&mut page_table, GlobalUserFlags::User);

        Self::from_components(
            ExecutionState::from_components(function, stack_size),
            page_table,
        )
    }

    pub fn switch_to(&self) -> ! {
        let satp = self.page_table.construct_satp(self.pid);

        unsafe {
            switch_to_user(
                core::ptr::addr_of!(*self.main_execution.trap_frame) as usize,
                self.main_execution.program_counter,
                satp,
            )
        }
    }
}
