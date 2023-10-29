#![allow(dead_code)]

use qor_core::{
    memory::{ByteCount, MemoryUnit},
    structures::id::PID,
};
use qor_riscv::memory::{
    mmu::{
        addresses::{PhysicalAddress, VirtualAddress},
        construct_satp,
        entry::{EntryPermissionFlags, GlobalUserFlags},
        table::PageTable,
    },
    PAGE_SIZE,
};

pub struct ManagedPageTable(PageTable);

static PAGE_ALLOC_FUNCTION: fn() -> *mut PageTable = || {
    crate::memory::get_page_bitmap_allocator()
        .allocate(1)
        .expect("Unable to allocate memory for page table mapping")
        .cast()
};

impl ManagedPageTable {
    /// Construct a [`PageTable`] filled with invalid entries
    #[must_use]
    pub const fn empty() -> Self {
        Self(PageTable::empty())
    }
}

impl core::default::Default for ManagedPageTable {
    fn default() -> Self {
        Self::empty()
    }
}

impl ManagedPageTable {
    /// Map a physical address to a virtual address in the given page table.
    ///
    /// # Panics
    ///
    /// This function will panic if there is not enough memory to allocate pages for the mapping.
    pub fn map(
        &mut self,
        virt_addr: VirtualAddress,
        phys_addr: PhysicalAddress,
        gu_flags: GlobalUserFlags,
        perm_flags: EntryPermissionFlags,
        level: usize,
    ) {
        // Safety:
        // The only safe methods for interacting with the inner `PageTable` are
        // by using the bitmap allocator to allocate pages, and the guarantees
        // from the `allocate` function give that it will be uniquely owned.
        unsafe {
            self.0.map(
                virt_addr,
                phys_addr,
                gu_flags,
                perm_flags,
                level,
                PAGE_ALLOC_FUNCTION,
            );
        };
    }

    /// Map a range of physical addresses to a range of virtual addresses.
    ///
    /// # Panics
    ///
    /// This function will panic if there is not enough memory to allocate pages for the mapping.
    pub fn map_range(
        &mut self,
        virt_addr: VirtualAddress,
        phys_addr: PhysicalAddress,
        range_length: MemoryUnit<PAGE_SIZE>,
        gu_flags: GlobalUserFlags,
        perm_flags: EntryPermissionFlags,
    ) {
        unsafe {
            self.0.map_range(
                virt_addr,
                phys_addr,
                range_length,
                gu_flags,
                perm_flags,
                PAGE_ALLOC_FUNCTION,
            );
        };
    }

    /// Map a range of physical addresses to the equivalent range of virtual addresses. The range ends just before the
    /// `end_addr`.
    ///
    /// # Panics
    ///
    /// This function will panic if `phys_addr` is not properly aligned, or if there is not enough memory to allocate
    /// to the mapping.
    pub fn id_map_range(
        &mut self,
        start_addr: PhysicalAddress,
        end_addr: PhysicalAddress,
        gu_flags: GlobalUserFlags,
        perm_flags: EntryPermissionFlags,
    ) {
        assert!(start_addr.inner() < end_addr.inner());

        #[allow(clippy::cast_possible_truncation)]
        self.map_range(
            VirtualAddress(start_addr.0),
            start_addr,
            ByteCount::new((end_addr.inner() - start_addr.inner()) as usize).convert(),
            gu_flags,
            perm_flags,
        );
    }

    /// Convert a virtual address to a physical address based on the mappings in this table, returning `None` if the
    /// mapping would produce an access violation.
    #[must_use]
    pub fn virtual_to_physical_address(
        &self,
        virt_addr: VirtualAddress,
    ) -> Option<PhysicalAddress> {
        self.0.virtual_to_physical_address(virt_addr)
    }

    /// Free all of the mapped pages in this table.
    ///
    /// # Panics
    ///
    /// This function will panic if any of the `PageTableEntries` in the `PageTable` are not properly allocated by the allocator.
    pub fn unmap_all(&mut self) {
        // Safety:
        // The passed `free_page` function is the pair to `PAGE_ALLOC_FUNCTION`
        // and can free any pages allocated by that function. The only safe way
        // for the page table to be interacted with is by using the
        // `PAGE_ALLOC_FUNCTION` as the allocator.
        unsafe {
            self.0.unmap_all(|ptr| {
                crate::memory::get_page_bitmap_allocator()
                    .free(ptr.cast(), 1)
                    .expect("Unable to free page from page table");
            });
        };
    }

    /// Set this page table as the currently used page table
    pub fn set_as_page_table(&'static mut self) {
        qor_riscv::memory::mmu::set_page_table(&mut self.0);
    }

    /// Construct a SATP from this page table
    #[must_use]
    pub fn construct_satp(&self, pid: PID) -> usize {
        construct_satp(pid.0, &self.0)
    }
}

/// Identity map the kernel to a `ManagedPageTable` stored on the heap
pub fn identity_map_kernel(table: &mut ManagedPageTable, gu_flags: GlobalUserFlags) {
    info!("Identity mapping kernel space");

    table.id_map_range(
        unsafe { crate::asm::HEAP_START }.into(),
        unsafe { crate::asm::HEAP_END }.into(),
        gu_flags,
        EntryPermissionFlags::ReadWrite,
    );

    table.id_map_range(
        unsafe { crate::asm::TEXT_START }.into(),
        unsafe { crate::asm::TEXT_END }.into(),
        gu_flags,
        EntryPermissionFlags::ReadExecute,
    );

    table.id_map_range(
        unsafe { crate::asm::RODATA_START }.into(),
        unsafe { crate::asm::RODATA_END }.into(),
        gu_flags,
        EntryPermissionFlags::ReadExecute,
    );

    table.id_map_range(
        unsafe { crate::asm::DATA_START }.into(),
        unsafe { crate::asm::DATA_END }.into(),
        gu_flags,
        EntryPermissionFlags::ReadWrite,
    );

    table.id_map_range(
        unsafe { crate::asm::BSS_START }.into(),
        unsafe { crate::asm::BSS_END }.into(),
        gu_flags,
        EntryPermissionFlags::ReadWrite,
    );

    table.id_map_range(
        unsafe { crate::asm::KERNEL_STACK_START }.into(),
        unsafe { crate::asm::KERNEL_STACK_END }.into(),
        gu_flags,
        EntryPermissionFlags::ReadWrite,
    );

    // UART PORT
    table.id_map_range(
        PhysicalAddress(0x1000_0000),
        PhysicalAddress(0x1000_1000),
        gu_flags,
        EntryPermissionFlags::ReadWrite,
    );

    // CLINT Device
    table.id_map_range(
        PhysicalAddress(0x200_0000),
        PhysicalAddress(0x201_0000),
        gu_flags,
        EntryPermissionFlags::ReadWrite,
    );

    // PLINT Device
    table.id_map_range(
        PhysicalAddress(0xc00_0000),
        PhysicalAddress(0xd00_0000),
        gu_flags,
        EntryPermissionFlags::ReadWrite,
    );    

    // Virt IO Devices
    table.id_map_range(
        PhysicalAddress(0x1000_0000),
        PhysicalAddress(0x1000_9000),
        gu_flags,
        EntryPermissionFlags::ReadWrite,
    );

    info!("Completed identity mapping kernel space");
}
