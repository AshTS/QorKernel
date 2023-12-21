#![allow(dead_code)]

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

/// Sequence of consecutively allocated pages from the global page bitmap allocator
pub struct PageSequence {
    ptr: core::ptr::NonNull<Page>,
    page_count: usize,
}

impl PageSequence {
    /// Get the pointer to the beginning of the page sequence
    #[must_use]
    pub const fn inner(&self) -> *mut Page {
        self.ptr.as_ptr()
    }

    /// Construct a new `PageSequence` from a length, allocating it on the `GLOBAL_PAGE_BITMAP_ALLOCATOR`
    pub fn alloc(length: usize) -> Self {
        let page_sequence = get_page_bitmap_allocator()
            .allocate(length)
            .expect("Unable to allocate page sequence");

        Self {
            ptr: core::ptr::NonNull::new(page_sequence).expect("Page sequence pointer is null"),
            page_count: length,
        }
    }

    /// Get the number of pages in the allocation
    #[must_use]
    pub const fn page_count(&self) -> usize {
        self.page_count
    }
}

impl core::ops::Drop for PageSequence {
    fn drop(&mut self) {
        // Safety: The only way to safely construct a `PageSequence` is from the `PageBitmapAllocator`
        unsafe {
            get_page_bitmap_allocator()
                .free(self.inner(), self.page_count)
                .expect("Unable to deallocate page sequence");
        }
    }
}

unsafe impl Sync for PageSequence {}
