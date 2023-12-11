use core::alloc::{GlobalAlloc, Layout};

use qor_core::memory::{allocators::byte::AllocationTable, ByteCount, MemoryUnit};
use qor_riscv::memory::PAGE_SIZE;

use crate::memory::get_page_bitmap_allocator;
use crate::memory::PAGE_BUMP_ALLOCATOR;

const TRACE_BYTE_ALLOC: bool = false;

#[global_allocator]
static GLOBAL_BYTE_ALLOCATOR: GlobalByteAllocatorWrapper = GlobalByteAllocatorWrapper::new();

pub struct GlobalByteAllocatorWrapper {
    inner: atomic_ref::AtomicRef<'static, AllocationTable>,
}

impl GlobalByteAllocatorWrapper {
    /// Construct a new, empty wrapper around an `AllocationTable` reference
    #[must_use]
    const fn new() -> Self {
        Self {
            inner: atomic_ref::AtomicRef::new(None),
        }
    }

    fn get(&self) -> &'static AllocationTable {
        self.inner
            .load(core::sync::atomic::Ordering::Acquire)
            .map_or_else(
                || {
                    error!("Global byte allocator not initialized");
                    panic!("Global byte allocator not initialized")
                },
                |v| v,
            )
    }

    /// Initialize the byte grained allocator with a certain amount of memory from the global page bump allocator.
    pub fn initialize(&self, memory_amount: MemoryUnit<{ PAGE_SIZE }>) {
        let mut allocation_table = AllocationTable::new();

        // Allocate enough pages to fulfill the requested amount of memory
        for _ in 0..memory_amount.raw() {
            allocation_table.add_region(
                PAGE_BUMP_ALLOCATOR
                    .allocate_object([0u8; PAGE_SIZE])
                    .unwrap(),
            );
        }

        let static_allocator_reference = PAGE_BUMP_ALLOCATOR
            .allocate_object(allocation_table)
            .unwrap();

        self.inner.store(
            Some(static_allocator_reference),
            core::sync::atomic::Ordering::Release,
        );
    }
}

unsafe impl GlobalAlloc for GlobalByteAllocatorWrapper {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = if layout.size() + layout.align() < 4096 {
            self.get()
                .alloc(layout.size(), layout.align())
                .expect("Unable to allocate memory via the byte allocator")
        } else if layout.align() <= PAGE_SIZE {
            get_page_bitmap_allocator()
                .allocate(ByteCount::new(layout.size()).convert::<PAGE_SIZE>().raw())
                .expect("Unable to allocate memory via the page bitmap allocator")
                .cast()
        } else {
            panic!("Unsupported layout {:?}", layout);
        };

        if TRACE_BYTE_ALLOC {
            trace!("ALLOC {:?} {:?}", ptr, layout);
        }

        assert!(
            ptr as usize % layout.align() == 0,
            "Alignment not satisfied"
        );

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if TRACE_BYTE_ALLOC {
            trace!("FREE {:?} {:?}", ptr, layout);
        }

        if layout.size() + layout.align() < 4096 {
            self.get().free(ptr as usize);
            self.get().coalesce_free_regions();
        } else if layout.align() <= PAGE_SIZE {
            get_page_bitmap_allocator()
                .free(
                    ptr.cast(),
                    ByteCount::new(layout.size()).convert::<PAGE_SIZE>().raw(),
                )
                .unwrap();
        } else {
            panic!("Unsupported layout {:?} on free", layout);
        }
    }
}

/// Initialize the global byte grained allocator with a certain amount of memory from the global page bump allocator
pub fn initialize_global_byte_allocator(memory_amount: MemoryUnit<{ PAGE_SIZE }>) {
    GLOBAL_BYTE_ALLOCATOR.initialize(memory_amount);

    info!(
        "Initialized global byte grained allocator with {} of memory",
        memory_amount
    );
}
