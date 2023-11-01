// Enable the ability to require functions to be four byte aligned
#![feature(fn_align)]
// Enable manual alignment checks
#![feature(pointer_is_aligned)]
#![no_std]
#![no_main]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

#[macro_use]
extern crate qor_core;

extern crate alloc;

mod asm;
mod drivers;
mod kprint;
mod memory;
mod panic;
mod process;
mod trap;

/// Entry point for the boot sequence, no interrupts are enabled when this function is called, and we are in machine
/// mode, no paging is enabled.
///
/// # Panics
/// This function will panic if a resource which is essential to the boot process is unavailable. For example, no UART
/// port being available, or there being insufficient memory to initialize the page table.
#[no_mangle]
#[repr(align(4))]
pub extern "C" fn kinit() {
    drivers::initialize_uart_driver().expect("Unable to initialize UART device driver");

    // Initialize the system logger to use the UART port
    kprint::assign_uart_logger();

    // Initialize the global page grained bump allocator
    memory::initialize_page_bump_allocator().expect("Unable to initialize bump allocator");

    // Initialize the global page grained bitmap allocator
    let dynamic_page_allocation_size = qor_core::memory::KiByteCount::new(1024);
    memory::initialize_page_bitmap_allocator(dynamic_page_allocation_size.convert())
        .expect("Unable to initialize bitmap allocator");

    // Construct page table which identity maps the kernel
    let page_table = memory::PAGE_BUMP_ALLOCATOR
        .allocate_object(memory::mmu::ManagedPageTable::empty())
        .expect("Unable to allocate space for root kernel page table");
    memory::mmu::identity_map_kernel(
        page_table,
        qor_riscv::memory::mmu::entry::GlobalUserFlags::None,
    );

    // Set the identity mapped page table as that used for the kernel in kmain
    page_table.set_as_page_table();

    // Initializing the trap frame
    crate::trap::initialize_trap_frame();

    // Note that by returning, we switch to supervisor mode, and move into `kmain`
}

/// Entry point for the core kernel functionality. Interrupts are enabled in this function, and we are in supervisor
/// mode, with paging enabled.
#[no_mangle]
#[repr(align(4))]
pub extern "C" fn kmain() {
    let hart_id = qor_core::structures::id::HartID::from(0);
    info!("Starting supervisor mode");

    // Initialize the byte grained allocator
    let byte_allocator_memory = qor_core::memory::KiByteCount::new(16);
    memory::initialize_global_byte_allocator(byte_allocator_memory.convert());

    // Set up the PLIC
    crate::drivers::initialize_plic(hart_id);
    info!("PLIC Initialized");

    // Initialize the CLINT timer
    crate::drivers::CLINT_DRIVER.set_frequency(qor_core::structures::time::Hertz(2));
    // crate::drivers::CLINT_DRIVER.start_timer(hart_id);
    info!("CLINT Initialized");

    // Probe the virt io address range
    info!("Starting VirtIO Device Discovery");
    crate::drivers::virtio::probe_virt_io_address_range();
    info!("VirtIO Device Discovery Complete");

    qor_core::tasks::execute_task(qor_core::tasks::Task::new(future()));
}

pub async fn future() {
    let mut buffer = alloc::boxed::Box::new([[0xffu8; 512]]);
    let result = drivers::get_block_driver().read_blocks(2, &mut *buffer);
    result.await.expect("Oops :(");
    core::mem::drop(buffer);
    kprint!("Read Complete\n");
}
