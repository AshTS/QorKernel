use qor_core::memory::MemoryUnit;

use crate::memory::{Page, PAGE_SIZE};

use super::{
    addresses::{PhysicalAddress, VirtualAddress},
    entry::{EntryPermissionFlags, GlobalUserFlags, PageTableEntry},
};

const LEVEL_SIZES: [usize; 3] = [4096, 4096 * 512, 4096 * 512 * 512];

/// Array of 512 [`PageTableEntry`]s which matches the size and alignment requirements of a [`Page`].
///
/// # Safety
/// For every valid entry in the [`PageTableEntry`], the `PhysicalAddress` from that entry is a valid pointer. If the
/// entry is a leaf, it is a valid pointer to a sequence of pages with the appropriate length. If the entry is not a
/// leaf, it is a valid pointer to a [`PageTable`].
#[repr(align(4096))]
#[derive(Clone, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)]
pub struct PageTable([PageTableEntry; 512]);

impl PageTable {
    /// Construct a [`PageTable`] filled with invalid entries
    #[must_use]
    pub const fn empty() -> Self {
        Self([PageTableEntry::invalid_entry(); 512])
    }
}

impl core::default::Default for PageTable {
    fn default() -> Self {
        Self([PageTableEntry::invalid_entry(); 512])
    }
}

static_assertions::assert_eq_align!(PageTable, Page);
static_assertions::assert_eq_size!(PageTable, Page);

impl PageTable {
    /// Map a physical address to a virtual address in the given page table
    ///
    /// # Safety
    ///
    /// The `alloc_page` function must return a pointer to a uniquely owned `PageTable` allocation.
    ///
    /// # Panics
    ///
    /// This function will panic if a valid [`PageTableEntry`] points to a null address.
    pub unsafe fn map(
        &mut self,
        virt_addr: VirtualAddress,
        phys_addr: PhysicalAddress,
        gu_flags: GlobalUserFlags,
        perm_flags: EntryPermissionFlags,
        level: usize,
        alloc_page: impl Fn() -> *mut Self,
    ) {
        // addr = self as *mut Self as usize;
        // trace!("Mapping {} pages at virtual {:x} to physical {:x} with permissions {}, {} in the page table at {:x}", LEVEL_SIZES[level] / PAGE_SIZE, virt_addr.inner(), phys_addr.inner(), gu_flags, perm_flags, addr);

        let mut walking_reference = &mut self.0[(virt_addr.vpn2() % 512) as usize];

        for level_index in (level..2).rev() {
            if !walking_reference.is_valid() {
                let allocated_page = alloc_page();
                let physical_allocated_page = PhysicalAddress(allocated_page as u64);

                *walking_reference = PageTableEntry::construct_valid(
                    [
                        physical_allocated_page.ppn0(),
                        physical_allocated_page.ppn1(),
                        physical_allocated_page.ppn2(),
                    ],
                    0,
                    GlobalUserFlags::None,
                    EntryPermissionFlags::None,
                );
            }

            // Safety:
            // Because this entry must be valid by the time we get here, we
            // have a valid pointer to the page table, because we have a
            // mutable reference to one `PageTable`, we also have unique access
            // to the pointers stored within it.
            let table_ref = (walking_reference.physical_address().0 as *mut Self)
                .as_mut()
                .unwrap();
            walking_reference = &mut table_ref.0[(virt_addr.vpn(level_index) % 512) as usize];
        }

        // Construct the new entry
        let entry = PageTableEntry::construct_valid(
            [phys_addr.ppn0(), phys_addr.ppn1(), phys_addr.ppn2()],
            0,
            gu_flags,
            perm_flags,
        );
        *walking_reference = entry;
    }

    /// Map a range of physical addresses to a range of virtual addresses
    ///
    /// # Safety
    ///
    /// The `alloc_page` function must return a pointer to a uniquely owned `PageTable` allocation.
    ///
    /// # Panics
    ///
    /// This function will panic if `virt_addr` or `phys_addr` is not properly aligned.
    pub unsafe fn map_range(
        &mut self,
        mut virt_addr: VirtualAddress,
        mut phys_addr: PhysicalAddress,
        mut range_length: MemoryUnit<PAGE_SIZE>,
        gu_flags: GlobalUserFlags,
        perm_flags: EntryPermissionFlags,
        alloc_page: impl Fn() -> *mut Self,
    ) {
        const LEVEL_0_MASK: u64 = 0xfff;
        const LEVEL_1_MASK: u64 = 0xff_ffff;
        const LEVEL_2_MASK: u64 = 0xf_ffff_ffff;

        let addr = self as *mut Self as usize;
        trace!("Mapping {} pages at virtual {:x} to physical {:x} with permissions {}, {} in the page table at {:x}", range_length.raw(), virt_addr.inner(), phys_addr.inner(), gu_flags, perm_flags, addr);

        while range_length.raw() > 0 {
            let level = if range_length.raw_bytes() >= LEVEL_SIZES[2]
                && virt_addr.inner() & LEVEL_2_MASK == 0
                && phys_addr.inner() & LEVEL_2_MASK == 0
            {
                2
            } else if range_length.raw_bytes() >= LEVEL_SIZES[1]
                && virt_addr.inner() & LEVEL_1_MASK == 0
                && phys_addr.inner() & LEVEL_1_MASK == 0
            {
                1
            } else if range_length.raw_bytes() >= LEVEL_SIZES[0]
                && virt_addr.inner() & LEVEL_0_MASK == 0
                && phys_addr.inner() & LEVEL_0_MASK == 0
            {
                0
            } else {
                panic!(
                    "map_range called with improperly aligned addresses PHYS: {:x}, VIRT: {:x}",
                    phys_addr.inner(),
                    virt_addr.inner()
                );
            };

            self.map(
                virt_addr,
                phys_addr,
                gu_flags,
                perm_flags,
                level,
                &alloc_page,
            );

            range_length = MemoryUnit::new(range_length.raw() - LEVEL_SIZES[level] / PAGE_SIZE);
            virt_addr.0 += LEVEL_SIZES[level] as u64;
            phys_addr.0 += LEVEL_SIZES[level] as u64;
        }
    }

    /// Map a range of physical addresses to the equivalent range of virtual addresses
    ///
    /// # Safety
    ///
    /// The `alloc_page` function must return a pointer to a uniquely owned `PageTable` allocation.
    ///
    /// # Panics
    ///
    /// This function will panic if `phys_addr` is not properly aligned.
    pub unsafe fn id_map_range(
        &mut self,
        phys_addr: PhysicalAddress,
        range_length: MemoryUnit<PAGE_SIZE>,
        gu_flags: GlobalUserFlags,
        perm_flags: EntryPermissionFlags,
        alloc_page: impl Fn() -> *mut Self,
    ) {
        self.map_range(
            VirtualAddress(phys_addr.0),
            phys_addr,
            range_length,
            gu_flags,
            perm_flags,
            alloc_page,
        );
    }

    /// Convert a virtual address to a physical address based on the mappings in this table.
    ///
    /// # Panics
    ///
    /// This function will panic if a valid [`PageTableEntry`] points to a null address.
    #[must_use]
    pub fn virtual_to_physical_address(
        &self,
        virt_addr: VirtualAddress,
    ) -> Option<PhysicalAddress> {
        let mut walking_reference = &self.0[(virt_addr.vpn2() % 512) as usize];

        for level_index in (0..=2).rev() {
            if !walking_reference.is_valid() {
                // This is an invalid entry, page fault
                break;
            } else if walking_reference.is_leaf() {
                let physical_page = walking_reference.ppn_level(level_index);
                return Some(PhysicalAddress(physical_page.0 | virt_addr.page_offset()));
            }

            // Safety:
            // Because this entry must be valid by the time we get here, we
            // have a valid pointer to the page table, because we have a
            // reference to one `PageTable`, we are able to safely construct a
            // reference to it.
            let table_ref =
                unsafe { (walking_reference.physical_address().0 as *mut Self).as_ref() }.unwrap();
            walking_reference = &table_ref.0[(virt_addr.vpn(level_index - 1) % 512) as usize];
        }

        None
    }

    /// Free all of the mapped pages in this table.
    ///
    /// # Safety
    ///
    /// The `free_page` function must be able to free a page allocated by the `alloc_page` function passed to the `map` functions.
    ///
    /// # Panics
    ///
    /// This function will panic if a valid [`PageTableEntry`] points to a null address.
    pub unsafe fn unmap_all(&mut self, free_page: impl Fn(*mut Self)) {
        for level2 in 0..512 {
            let entry2 = &mut self.0[level2];
            if entry2.is_valid() && !entry2.is_leaf() {
                // Safety:
                // Because this entry must be valid by the time we get here, we
                // have a valid pointer to the page table, because we have a
                // reference to one `PageTable`, we are able to safely construct a
                // mutable reference to it as we have a mutable reference to the
                // entire table.
                let level1_table = unsafe {
                    (entry2.physical_address().inner() as *mut Self)
                        .as_mut()
                        .unwrap()
                };
                *entry2 = PageTableEntry::invalid_entry();

                for level1 in 0..512 {
                    let entry1 = &mut level1_table.0[level1];
                    if entry1.is_valid() && !entry1.is_leaf() {
                        // Safety:
                        // This address must be a valid page that can be freed
                        // by `free_page` due to the safety requirements of `unmap_all`.
                        let address = entry1.physical_address().inner() as *mut Self;
                        *entry1 = PageTableEntry::invalid_entry();
                        free_page(address);
                    }
                }

                // Safety:
                // We can now free this address, as the safety requirements of `unmap_all` requires `free_page` can safely be used to free this memory.
                free_page(level1_table as *mut Self);
            }
        }
    }
}
