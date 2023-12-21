use core::sync::atomic::AtomicUsize;

use qor_core::{memory::allocators::page::bitmap::{PageBox, AllocationError}, structures::mem::PermissionFlags};
use qor_riscv::memory::{Page, mmu::addresses::VirtualAddress, PageCount, PAGE_SIZE};

use crate::memory::{get_page_bitmap_allocator, PageSequence, mmu::ManagedPageTable};

pub struct ProcessBox<'allocator, Page: 'static, T> {
    inner: PageBox<'allocator, Page, T>,
    stat_tracker: alloc::sync::Weak<MemoryStatistics>
}

impl<'allocator, Page: 'static, T> core::ops::Drop for ProcessBox<'allocator, Page, T> {
    fn drop(&mut self) {
        if let Some(tracker) = self.stat_tracker.upgrade() {
            tracker.resident.fetch_sub(self.inner.page_count(), core::sync::atomic::Ordering::AcqRel);
        }
    }
}

pub struct ProcessPageSequence {
    inner: PageSequence,
    stat_tracker: alloc::sync::Weak<MemoryStatistics>
}

impl core::ops::Drop for ProcessPageSequence {
    fn drop(&mut self) {
        if let Some(tracker) = self.stat_tracker.upgrade() {
            tracker.resident.fetch_sub(self.inner.page_count(), core::sync::atomic::Ordering::AcqRel);
        }
    }
}

pub struct MappedPageSequence {
    permissions: PermissionFlags,
    virtual_address: VirtualAddress,
    inner: PageSequence,
    stat_tracker: alloc::sync::Weak<MemoryStatistics>
}

impl core::ops::Drop for MappedPageSequence {
    fn drop(&mut self) {
        if let Some(tracker) = self.stat_tracker.upgrade() {
            tracker.resident.fetch_sub(self.inner.page_count(), core::sync::atomic::Ordering::AcqRel);
        }
    }
}


#[allow(clippy::module_name_repetitions)]
pub struct MemoryStatistics {
    size: AtomicUsize,
    resident: AtomicUsize,
    shared: AtomicUsize,
}

impl MemoryStatistics {
    pub const fn new() -> Self {
        Self {
            size: AtomicUsize::new(0),
            resident: AtomicUsize::new(0),
            shared: AtomicUsize::new(0),
        }
    }

    pub fn size(&self) -> usize {
        self.size.load(core::sync::atomic::Ordering::Acquire)
    }

    pub fn resident(&self) -> usize {
        self.resident.load(core::sync::atomic::Ordering::Acquire)
    }

    pub fn shared(&self) -> usize {
        self.shared.load(core::sync::atomic::Ordering::Acquire)
    }

    pub fn alloc_page_box<T>(self: &alloc::sync::Arc<Self>, value: T) -> Result<ProcessBox<'static, Page, T>, AllocationError> {
        let result = get_page_bitmap_allocator().alloc_boxed(value).map(|inner| ProcessBox {
            inner,
            stat_tracker: alloc::sync::Arc::downgrade(&self.clone()),
        });

        if let Ok(value) = &result {
            self.resident.fetch_add(value.inner.page_count(), core::sync::atomic::Ordering::AcqRel);
        } 

        result
    }

    pub fn alloc_page_sequence(self: &alloc::sync::Arc<Self>, length: PageCount) -> ProcessPageSequence {
        let inner = PageSequence::alloc(length.raw());
        self.resident.fetch_add(inner.page_count(), core::sync::atomic::Ordering::AcqRel);

        ProcessPageSequence {
            inner,
            stat_tracker: alloc::sync::Arc::downgrade(&self.clone()),
        }
    }

    pub fn map_page_sequence(self: &alloc::sync::Arc<Self>, page_table: &mut ManagedPageTable, length: PageCount, virtual_address: VirtualAddress, permissions: PermissionFlags) -> MappedPageSequence {
        let inner = PageSequence::alloc(length.raw());
        self.resident.fetch_add(inner.page_count(), core::sync::atomic::Ordering::AcqRel);

        page_table.map_range(virtual_address, inner.inner().into(), length, qor_riscv::memory::mmu::entry::GlobalUserFlags::User, permissions.try_into().expect("Unable to convert permission flags"));

        MappedPageSequence {
            permissions,
            virtual_address,
            inner,
            stat_tracker: alloc::sync::Arc::downgrade(&self.clone()),
        }
    }
}

impl<'allocator, Page: 'static, T> core::ops::Deref for ProcessBox<'allocator, Page, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'allocator, Page: 'static, T> core::ops::DerefMut for ProcessBox<'allocator, Page, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl core::ops::Deref for MappedPageSequence {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        // TODO: Add Safety Justification
        unsafe { core::slice::from_raw_parts(self.inner.inner().cast(), self.inner.page_count() * PAGE_SIZE) }
    }
}

impl core::ops::DerefMut for MappedPageSequence {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // TODO: Add Safety Justification
        unsafe { core::slice::from_raw_parts_mut(self.inner.inner().cast(), self.inner.page_count() * PAGE_SIZE) }
    }
}