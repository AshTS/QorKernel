use qor_core::memory::allocators::page::bump::PageBumpAllocator;
use qor_riscv::memory::Page;

use crate::asm;

/// Global page grained bump allocator.
pub static PAGE_BUMP_ALLOCATOR: PageBumpAllocator<Page> = PageBumpAllocator::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocatorInitializationError;

/// Initialize the global page grained bump allocator.
pub fn initialize_page_bump_allocator() -> Result<(), AllocatorInitializationError> {
    // Access the heap start and end variables to store them locally
    // Safety: These are just constants being read
    let heap_start = unsafe { asm::HEAP_START };
    let heap_end: *mut Page = unsafe { asm::HEAP_END };

    // Manually check the safety requirements of the `assign_region` function.
    if heap_start.is_null() {
        error!("Heap start is a null pointer.");
        return Err(AllocatorInitializationError);
    } else if heap_end.is_null() {
        error!("Heap end is a null pointer.");
        return Err(AllocatorInitializationError);
    } else if !heap_start.is_aligned() {
        error!("Heap start is unaligned: {:x?}.", heap_start);
        return Err(AllocatorInitializationError);
    } else if !heap_end.is_aligned() {
        error!("Heap end is unaligned: {:x?}.", heap_end);
        return Err(AllocatorInitializationError);
    } else if heap_start >= heap_end {
        error!("Heap start and end are in the wrong order (start: {:x?}, end: {:x?}), unable to initialize bump allocator.", heap_start, heap_end);
        return Err(AllocatorInitializationError);
    }

    // Safety:
    // Non-null, aligned, and proper ordering are checked manually by the above code.
    // Valid range is given by the definition of these symbols in the linker script, and them only being used here.s
    unsafe {
        PAGE_BUMP_ALLOCATOR.assign_region(asm::HEAP_START..asm::HEAP_END);
    }

    info!("Initialized global page grained bump allocator.");

    Ok(())
}
