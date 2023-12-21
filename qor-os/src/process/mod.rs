#![allow(dead_code)]

use core::{sync::atomic::AtomicU16, ops::DerefMut};

use alloc::sync::Arc;
use qor_core::{structures::{id::{ProcessID, PID}, elf::Elf, mem::{PermissionFlags, PermissionFlag}, syscall_error::SyscallError}, memory::ByteCount, interfaces::fs::FileDescriptor};
use qor_riscv::{
    memory::{mmu::{entry::GlobalUserFlags, addresses::VirtualAddress}, Page, PageCount, PAGE_SIZE},
    trap::frame::TrapFrame,
};

use crate::{
    memory::mmu::ManagedPageTable,
    trap::allocate_trap_frame, syscalls::structures::UserspaceAddress,
};

use self::{memory::{MemoryStatistics, ProcessBox, MappedPageSequence}, proc_interface::ProcessData};

pub mod boxed;
pub mod memory;
pub mod proc_interface;

static PID_COUNTER: AtomicU16 = AtomicU16::new(1);

type ProgramTableMutex = qor_core::sync::Mutex<alloc::collections::BTreeMap<PID, Process>>;
static PROGRAM_TABLE: ProgramTableMutex = qor_core::sync::Mutex::new(alloc::collections::BTreeMap::new());

/// Get the next PID to be used
fn new_pid() -> PID {
    ProcessID(PID_COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed))
}

/// Execution state for process execution. Includes a trap frame (which doesn't store the information for executing
/// traps, but for executing user mode), a program counter storing where in the executable we return to, and a sequence
/// of pages used for the stack.
pub struct ExecutionState {
    program_counter: usize,
    stack: MappedPageSequence,
    trap_frame: ProcessBox<'static, Page, TrapFrame>,
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
    page_table: ProcessBox<'static, Page, ManagedPageTable>,
    memory_stats: alloc::sync::Arc<MemoryStatistics>,
    mapped_pages: alloc::vec::Vec<MappedPageSequence>,
    interface_data: ProcessData
}

impl ExecutionState {
    pub fn from_components(memory_stats: &alloc::sync::Arc<MemoryStatistics>, page_table: &mut ManagedPageTable, initial_program_counter: usize, stack_size: PageCount) -> Self {
        let mut trap_frame = memory_stats
            .alloc_page_box(allocate_trap_frame())
            .expect("Unable to allocate trap frame space");
        
        let stack_addr = 0x1_0000_0000;

        let stack = memory_stats.map_page_sequence(page_table, stack_size, VirtualAddress(stack_addr), PermissionFlags::new(0) | PermissionFlag::Read | PermissionFlag::Write);
        trap_frame.registers[2] = stack_addr + stack_size.raw_bytes() as u64;

        Self {
            program_counter: initial_program_counter,
            stack,
            trap_frame
        }
    }
}

extern "C" {
    pub fn switch_to_user(frame: usize, pc: usize, satp: usize) -> !;
}

impl Process {
    pub fn from_components(
        execution_state: ExecutionState,
        page_table: ProcessBox<'static, Page, ManagedPageTable>,
        memory_stats: alloc::sync::Arc<MemoryStatistics>,
    ) -> Self {
        Self {
            pid: new_pid(),
            main_execution: execution_state,
            state: ProcessState::Active,
            page_table,
            memory_stats,
            mapped_pages: alloc::vec::Vec::new(),
            interface_data: ProcessData::new()
        }
    }

    pub fn from_fn_ptr(function: usize, stack_size: PageCount) -> Self {
        let mem_stats = alloc::sync::Arc::new(MemoryStatistics::new());

        let mut page_table = mem_stats.alloc_page_box(crate::memory::mmu::ManagedPageTable::empty())
            .expect("Unable to allocate space for process page table");
        crate::memory::mmu::identity_map_kernel(&mut page_table, GlobalUserFlags::User);

        Self::from_components(
            ExecutionState::from_components(&mem_stats, &mut page_table, function, stack_size),
            page_table,
            mem_stats
        )
    }

    pub fn from_elf_file(elf: Elf<'_>, stack_size: PageCount) -> Self {
        let mem_stats = alloc::sync::Arc::new(MemoryStatistics::new());

        let mut page_table = mem_stats.alloc_page_box(crate::memory::mmu::ManagedPageTable::empty())
            .expect("Unable to allocate space for process page table");
        crate::memory::mmu::identity_map_kernel(&mut page_table, GlobalUserFlags::User);

        let mut proc = Self::from_components(ExecutionState::from_components(&mem_stats, &mut page_table, elf.header.entry.try_into().unwrap(), stack_size), page_table, mem_stats);
    
        for program_header in elf.program_headers {
            if program_header.header_type == qor_core::structures::elf::enums::ProgramHeaderType::Load {
                let permissions: PermissionFlags = program_header.flags.into();
                let virtual_address = VirtualAddress(program_header.virtual_addr & !(PAGE_SIZE as u64 - 1));
                let page_offset: usize = (program_header.virtual_addr & (PAGE_SIZE as u64 - 1)).try_into().unwrap();
                let file_length: usize = program_header.file_size.try_into().unwrap();
                let length = ByteCount::new(program_header.memory_size.try_into().unwrap()).convert();
                let file_offset = program_header.offset.try_into().unwrap();

                let sequence = proc.map_page_sequence(virtual_address, length, permissions);
                sequence.deref_mut()[page_offset..page_offset + file_length].copy_from_slice(&elf.data[file_offset.. file_offset + file_length]);
            } 
        }
        
        proc
    }

    pub fn map_page_sequence(&mut self, virtual_address: VirtualAddress, length: PageCount, permissions: PermissionFlags) -> &mut MappedPageSequence {
        let sequence = self.memory_stats.map_page_sequence(&mut self.page_table, length, virtual_address, permissions);
        self.mapped_pages.push(sequence);

        self.mapped_pages.last_mut().unwrap()
    }

    pub fn get_switching_data(&self) -> (usize, usize, usize) {
        (self.page_table.construct_satp(self.pid), core::ptr::addr_of!(*self.main_execution.trap_frame) as usize, self.main_execution.program_counter)
    }

    pub fn switch(data: (usize, usize, usize)) -> ! {
        unsafe {
            switch_to_user(
                data.1,
                data.2,
                data.0,
            )
        }
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

    pub fn kernel_pointer(&self, address: UserspaceAddress) -> Result<usize, SyscallError> {
        self.page_table.virtual_to_physical_address(VirtualAddress(address.0.try_into().unwrap())).map(|v| v.0.try_into().unwrap()).ok_or(SyscallError::Fault)
    }

    pub fn file_descriptor(&self, descriptor: usize) -> Result<&Arc<dyn FileDescriptor>, SyscallError> {
        self.interface_data.file_descriptors.get(&descriptor).ok_or(SyscallError::BadFileDescriptor)
    }

    pub fn registers(&self) -> &[u64; 32] {
        &self.main_execution.trap_frame.registers
    }

    pub fn registers_mut(&mut self) -> &mut [u64; 32] {
        &mut self.main_execution.trap_frame.registers
    }
}

#[allow(clippy::module_name_repetitions)]
pub fn start_process(proc: Process) {
    PROGRAM_TABLE.spin_lock().insert(proc.pid, proc);
}

pub fn processes() -> &'static ProgramTableMutex {
    &PROGRAM_TABLE
}