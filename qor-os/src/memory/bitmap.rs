use qor_core::memory::{
    allocators::page::{bitmap::PageBitmapAllocator, bump::AllocationError},
    MemoryUnit,
};

use qor_riscv::memory::Page;

use crate::memory::PAGE_BUMP_ALLOCATOR;

/// Global page grained bitmap allocator.
pub static PAGE_BITMAP_ALLOCATOR: atomic_ref::AtomicRef<'static, PageBitmapAllocator<Page>> =
    atomic_ref::AtomicRef::new(None);

/// Get a reference to the global page bitmap allocator if it exists, otherwise panic with an appropriate error
#[allow(dead_code)]
pub fn get_page_bitmap_allocator() -> &'static PageBitmapAllocator<Page> {
    PAGE_BITMAP_ALLOCATOR
        .load(core::sync::atomic::Ordering::Acquire)
        .map_or_else(
            || {
                error!("Global page bitmap allocator not initialized");
                panic!("Global page bitmap allocator not initialized")
            },
            |v| v,
        )
}

/// Initialize the global page bitmap allocator with a certain amount of memory from the global page bump allocator
pub fn initialize_page_bitmap_allocator(
    memory_amount: MemoryUnit<{ qor_riscv::memory::PAGE_SIZE }>,
) -> Result<(), AllocationError> {
    let alloted_memory = PAGE_BUMP_ALLOCATOR.allocate(memory_amount.raw())?;
    let bitmap_allocator = PageBitmapAllocator::from_pages(alloted_memory);
    let static_allocator_reference = PAGE_BUMP_ALLOCATOR.allocate_object(bitmap_allocator)?;

    PAGE_BITMAP_ALLOCATOR.store(
        Some(static_allocator_reference),
        core::sync::atomic::Ordering::Release,
    );

    info!(
        "Initialized global page grained bitmap allocator with {} of memory",
        memory_amount
    );
    Ok(())
}
