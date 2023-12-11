use core::{
    mem::size_of,
    sync::atomic::{AtomicPtr, AtomicU32, Ordering},
};

const LOCKED_BIT_MASK: u32 = 0x8000_0000;
const VALID_BIT_MASK: u32 = 0x4000_0000;
const ALLOCATED_BIT_MASK: u32 = 0x2000_0000;
const ALLOCATION_LENGTH_MASK: u32 = 0x1FFF_0000;
const NEXT_ENTRY_INDEX_MASK: u32 = 0x0000_FFFF;

/// The `AllocationTableEntry`is a structure that represents a single entry into the byte allocator table. The data contained needs to include some bit flags.
/// ```
/// +--------+-------+-----------+-------------------+------------------+--------------------------+
/// | 1 Bit  | 1 Bit |   1 Bit   |      13 Bits      |     16 Bits      |         32 Bits          |
/// +--------+-------+-----------+-------------------+------------------+--------------------------+
/// | Locked | Valid | Allocated | Allocation Length | Next Entry Index | Lower 32 Bits of Pointer |
/// +--------+-------+-----------+-------------------+------------------+--------------------------+
/// ```
///
/// - The locked bit is used to denote that the entry is locked (and thus being used by another thread).
/// - The valid bit is used to denote if the entry is valid and thus contains a reference to an allocation.
/// - The allocated bit is used to denote that the entry is referring to an allocated section of memory.
/// - The allocation length is the number of bytes contained in the allocation.
/// - The next entry index is the index into the allocation tables of the next entry.
/// - The lower 32 bits of the pointer is combined with the upper 32 bits of the pointer from the `AllocationTable` to
///   form a full pointer to the allocation.
#[derive(Debug)]
pub struct AllocationTableEntry {
    flags: AtomicU32,
    ptr_offset: AtomicU32,
}

impl AllocationTableEntry {
    /// Construct an empty [`AllocationTableEntry`], which is initialized as unlocked, invalid, and free.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            flags: AtomicU32::new(0),
            ptr_offset: AtomicU32::new(0),
        }
    }

    /// Attempts to lock this [`AllocationTableEntry`].
    pub fn lock(&self) -> Option<AllocationTableEntryGuard> {
        if self.flags.fetch_or(LOCKED_BIT_MASK, Ordering::Acquire) & LOCKED_BIT_MASK == 0 {
            Some(AllocationTableEntryGuard { entry: self })
        } else {
            None
        }
    }
}

pub struct AllocationTableEntryGuard<'a> {
    entry: &'a AllocationTableEntry,
}

impl<'a> AllocationTableEntryGuard<'a> {
    /// Releases the lock on this [`AllocationTableEntry`].
    ///
    /// # Safety
    ///
    /// This `AllocationTableEntryGuard` must never be modified after this release.
    unsafe fn release_lock(&mut self) {
        self.entry
            .flags
            .fetch_and(!LOCKED_BIT_MASK, Ordering::Release);
    }

    /// Get the next pointer for this entry.
    #[must_use]
    pub fn next(&self) -> u16 {
        (self.entry.flags.load(Ordering::Relaxed) & NEXT_ENTRY_INDEX_MASK) as u16
    }

    /// Set the next pointer for this entry.
    pub fn set_next(&self, next_pointer: u16) {
        self.entry
            .flags
            .fetch_and(!NEXT_ENTRY_INDEX_MASK, Ordering::Relaxed);
        self.entry
            .flags
            .fetch_or(u32::from(next_pointer), Ordering::Relaxed);
    }

    /// Get the lower 32 bits of the pointer for this entry
    #[must_use]
    pub fn low_pointer(&self) -> u32 {
        self.entry.ptr_offset.load(Ordering::Relaxed)
    }

    /// Set the lower 32 bits of the pointer for this entry
    pub fn set_low_pointer(&self, lower: u32) {
        self.entry.ptr_offset.store(lower, Ordering::Relaxed);
    }

    /// Get the full pointer for this entry given the upper 32 bits
    #[must_use]
    pub fn pointer(&self, upper: u32) -> usize {
        ((upper as usize) << 32) | (self.low_pointer() as usize)
    }

    /// Get the allocation length
    #[must_use]
    pub fn allocation_length(&self) -> usize {
        ((self.entry.flags.load(Ordering::Relaxed) & ALLOCATION_LENGTH_MASK) >> 16) as usize
    }

    /// Set the allocation length
    pub fn set_allocation_length(&self, length: usize) {
        self.entry
            .flags
            .fetch_and(!ALLOCATION_LENGTH_MASK, Ordering::Relaxed);
        self.entry.flags.fetch_or(
            u32::try_from(length & 0x1FFF).expect("Length Too Big") << 16,
            Ordering::Relaxed,
        );
    }

    /// Get the valid flag
    #[must_use]
    pub fn valid(&self) -> bool {
        (self.entry.flags.load(Ordering::Relaxed) & VALID_BIT_MASK) > 0
    }

    /// Get the allocated flag
    #[must_use]
    pub fn allocated(&self) -> bool {
        (self.entry.flags.load(Ordering::Relaxed) & ALLOCATED_BIT_MASK) > 0
    }

    /// Set the valid flag
    pub fn set_valid(&self, value: bool) {
        if value {
            self.entry.flags.fetch_or(VALID_BIT_MASK, Ordering::Relaxed);
        } else {
            self.entry
                .flags
                .fetch_and(!VALID_BIT_MASK, Ordering::Relaxed);
        }
    }

    /// Set the allocated flag
    pub fn set_allocated(&self, value: bool) {
        if value {
            self.entry
                .flags
                .fetch_or(ALLOCATED_BIT_MASK, Ordering::Relaxed);
        } else {
            self.entry
                .flags
                .fetch_and(!ALLOCATED_BIT_MASK, Ordering::Relaxed);
        }
    }

    /// Update the data within the `AllocationTableEntry`
    pub fn update(&self, valid: bool, allocated: bool, size: usize, pointer_low: u32) {
        self.set_valid(valid);
        self.set_allocated(allocated);
        self.set_allocation_length(size);
        self.set_low_pointer(pointer_low);
    }
}

impl<'a> core::ops::Drop for AllocationTableEntryGuard<'a> {
    fn drop(&mut self) {
        // Safety: We are in the drop implementation, thus this guard will never be used again.
        unsafe { self.release_lock() }
    }
}

const ALLOCATION_TABLE_LENGTH: usize =
    (4096 - 4 * size_of::<usize>()) / size_of::<AllocationTableEntry>();

#[repr(align(4096))]
#[derive(Debug)]
pub struct AllocationTable {
    // Guarantees:
    // - If the `previous` pointer is non-null, it must be a valid pointer to an `AllocationTable` with a `next` pointer that points to this table.
    // - If the `next` pointer is non-null, it must be a valid pointer to an `AllocationTable`, with a `previous` pointer that points to this table.
    // - If there is a `previous` table, its `start_index` must be this table's `start_index - ALLOCATION_TABLE_LENGTH`.
    // - If there is a `next` table, its `start_index` must be this table's `start_index + ALLOCATION_TABLE_LENGTH`.
    previous: AtomicPtr<AllocationTable>, // Treat like: Option<&'static mut AllocationTable>
    next: AtomicPtr<AllocationTable>,     // Treat like: Option<&'static mut AllocationTable>
    pointer_upper_32: u32,
    start_index: usize,
    entries: [AllocationTableEntry; ALLOCATION_TABLE_LENGTH],
}

impl AllocationTable {
    /// Set the previous pointer of this page, and the next pointer of the page being linked to. Modifies the `start_index` of the *previous* table.
    pub fn set_previous(&'static self, previous: &'static mut Self) {
        previous
            .next
            .store(self as *const Self as *mut Self, Ordering::Relaxed);
        previous.start_index = self.start_index - ALLOCATION_TABLE_LENGTH;
        self.previous
            .store(previous as *mut Self, Ordering::Relaxed);
    }

    /// Set the next pointer of this page, and the previous pointer of the page being linked to. Modifies the `start_index` of the *next* table.
    pub fn set_next(&'static self, next: &'static mut Self) {
        next.previous
            .store(self as *const Self as *mut Self, Ordering::Relaxed);
        next.start_index = self.start_index + ALLOCATION_TABLE_LENGTH;
        self.next.store(next as *mut Self, Ordering::Relaxed);
    }

    /// Construct a static reference from the previous pointer.
    pub fn previous_ref(&self) -> Option<&'static Self> {
        let prev = self.previous.load(Ordering::Relaxed);

        // Safety:
        // - The only safe way to get a non-null pointer inserted into this
        // field is via the `set_previous` function which requires a valid
        // static reference.
        unsafe { prev.as_ref() }
    }

    /// Construct a static reference from the next pointer.
    pub fn next_ref(&self) -> Option<&'static Self> {
        let next = self.next.load(Ordering::Relaxed);

        // Safety:
        // - The only safe way to get a non-null pointer inserted into this
        // field is via the `set_previous` function which requires a valid
        // static reference.
        unsafe { next.as_ref() }
    }

    /// Construct a new, empty [`AllocationTable`].
    #[must_use]
    pub const fn new() -> Self {
        // Justification: This is only for zero initializing the allocation table.
        #[allow(clippy::declare_interior_mutable_const)]
        const EMPTY: AllocationTableEntry = AllocationTableEntry::empty();

        Self {
            previous: AtomicPtr::new(core::ptr::null_mut()),
            next: AtomicPtr::new(core::ptr::null_mut()),
            pointer_upper_32: 0,
            start_index: 0,
            entries: [EMPTY; ALLOCATION_TABLE_LENGTH],
        }
    }

    /// Construct a new [`AllocationTable`] with a region of memory available for allocation.
    ///
    /// # Panics
    ///
    /// This function will panic if it is somehow unable to lock table entries.
    #[must_use]
    pub fn construct_with_region(region: &'static mut [u8]) -> Self {
        let mut table = Self::new();
        table.add_region(region);

        table
    }

    /// Add a region of memory to be available from the allocator.
    ///
    /// # Panics
    ///
    /// This function will panic if it is unable to add a region due to being blocked by another thread.
    pub fn add_region(&mut self, region: &'static mut [u8]) {
        let pointer = region.as_mut_ptr() as u64;

        let low = (pointer & 0xffff_ffff) as u32;
        let high = (pointer >> 32) as u32;

        let mut current = self
            .index(0)
            .expect("Need to be able to access the first entry");
        let guard = current.lock().unwrap();
        #[allow(clippy::if_not_else)]
        if !guard.valid() {
            guard.update(true, false, region.len(), low);
            guard.set_next(0);
            core::mem::drop(guard);
            self.pointer_upper_32 = high;
        } else {
            core::mem::drop(guard);
            loop {
                if let Some(guard) = current.lock() {
                    assert!(guard.valid());
                    let next_index = guard.next();
                    if next_index > 0 {
                        current = self.index(next_index as usize).expect("Bad Link");
                    } else {
                        let (free_guard_index, _, free_guard) = self
                            .find_first_invalid(region.as_mut_ptr() as usize)
                            .unwrap();
                        free_guard.set_next(0);
                        free_guard.update(true, false, region.len(), low);
                        guard.set_next(free_guard_index);
                        break;
                    }
                } else {
                    todo!();
                }
            }
        }

        assert_eq!(self.pointer_upper_32, high);
    }

    /// Get a reference to the entry at the given index, walking the linked list of `AllocationTable`s if necessary.
    pub fn index(&self, index: usize) -> Option<&AllocationTableEntry> {
        if index < self.start_index {
            self.previous_ref()?.index(index)
        } else if index >= self.start_index + ALLOCATION_TABLE_LENGTH {
            self.next_ref()?.index(index)
        } else {
            let shifted_index = index - self.start_index;
            Some(&self.entries[shifted_index])
        }
    }

    /// Find first invalid entry with the right upper 32 bits with regards to the pointer passed in.
    ///
    /// # Panics
    ///
    /// This function will panic if the indexes exceed the allotted space.
    pub fn find_first_invalid(&self, ptr: usize) -> Option<(u16, u32, AllocationTableEntryGuard)> {
        if self.pointer_upper_32 == (ptr as u64 >> 32) as u32 {
            for index in 0..ALLOCATION_TABLE_LENGTH {
                if let Some(guard) = self.entries[index].lock() {
                    if !guard.valid() {
                        return Some((
                            (index + self.start_index).try_into().unwrap(),
                            self.pointer_upper_32,
                            guard,
                        ));
                    }
                }
            }

            None
        } else {
            self.next_ref()?.find_first_invalid(ptr)
        }
    }

    /// Debug dump the allocation table
    pub fn debug_dump_all(&self) {
        let mut last_valid = 0;
        for index in 0..ALLOCATION_TABLE_LENGTH {
            let entry = &self.entries[index];
            #[allow(clippy::nursery)]
            if let Some(guard) = entry.lock() {
                if guard.valid() {
                    if index - last_valid > 3 {
                        debug!("...");
                        debug!("{:5}: Invalid", index + self.start_index - 1);
                    }

                    last_valid = index;
                    let size = guard.allocation_length();
                    let address = guard.pointer(self.pointer_upper_32);
                    if guard.next() == 0 {
                        debug!(
                            "{:5}: [{}] {} Byte{} at {:#x} ->|",
                            index + self.start_index,
                            if guard.allocated() { "ALLOC" } else { "FREE " },
                            size,
                            if size == 1 { "" } else { "s" },
                            address
                        );
                    } else {
                        debug!(
                            "{:5}: [{}] {} Byte{} at {:#x} -> {}",
                            index + self.start_index,
                            if guard.allocated() { "ALLOC" } else { "FREE " },
                            size,
                            if size == 1 { "" } else { "s" },
                            address,
                            guard.next()
                        );
                    }
                } else if last_valid + 1 == index || index == ALLOCATION_TABLE_LENGTH - 1 {
                    if index == ALLOCATION_TABLE_LENGTH - 1 {
                        debug!("       ...");
                    }
                    debug!("{:5}: Invalid", index + self.start_index);
                }
            } else {
                debug!("{:5}: Busy", index + self.start_index);
            }
        }
    }

    /// Search for a region of memory with the given alignment and size.
    ///
    /// # Panics
    ///
    /// This function will panic if it encounters an invalid state.
    pub fn alloc(&self, size: usize, align: usize) -> Option<*mut u8> {
        let mut current = self.index(0)?;
        loop {
            if let Some(guard) = current.lock() {
                assert!(guard.valid());

                if !guard.allocated() {
                    let low_ptr = guard.low_pointer() as usize;
                    let align_slack = (align - (low_ptr % align)) % align;
                    #[allow(clippy::comparison_chain)]
                    if guard.allocation_length() == align_slack + size {
                        guard.set_allocated(true);
                        return Some(
                            (guard.pointer(self.pointer_upper_32) + align_slack) as *mut u8,
                        );
                    } else if guard.allocation_length() > align_slack + size {
                        if let Some((free_index, upper_32, free)) = self.find_first_invalid(
                            guard.pointer(self.pointer_upper_32) + align_slack + size,
                        ) {
                            free.update(
                                true,
                                false,
                                guard.allocation_length() - align_slack - size,
                                guard
                                    .low_pointer()
                                    .wrapping_add((align_slack + size).try_into().unwrap()),
                            );
                            free.set_next(guard.next());

                            guard.update(true, true, align_slack + size, guard.low_pointer());
                            guard.set_next(free_index);

                            return Some((guard.pointer(upper_32) + align_slack) as *mut u8);
                        }
                    }
                }

                let next_index = guard.next();
                if next_index > 0 {
                    current = self.index(next_index as usize)?;
                } else {
                    break;
                }
            } else {
                return None;
            }
        }

        None
    }

    /// Free a region of allocated memory. Note that this function does not assume that the pointer given is at the start of the memory region, only that it is within the region.
    ///
    /// # Panics
    ///
    /// This function will panic if it encounters an invalid state.
    pub fn free(&self, ptr: usize) {
        let mut current = self
            .index(0)
            .expect("Need to be able to access the first entry");
        loop {
            if let Some(guard) = current.lock() {
                assert!(guard.valid());

                if guard.allocated() {
                    let ptr_start = guard.pointer(self.pointer_upper_32);
                    let length = guard.allocation_length();

                    if ptr >= ptr_start && ptr < ptr_start + length {
                        guard.set_allocated(false);
                        return;
                    }
                }

                let next_index = guard.next();
                if next_index > 0 {
                    current = self.index(next_index as usize).expect("Bad Link");
                } else {
                    break;
                }
            } else {
                todo!();
            }
        }

        panic!()
    }

    /// Coalesce sequential free regions
    ///
    /// # Panics
    ///
    /// This function will panic if it reaches an invalid state.
    pub fn coalesce_free_regions(&self) {
        let mut current = self.index(0).expect("Unable to access first entry");
        loop {
            if let Some(guard) = current.lock() {
                assert!(guard.valid());

                let next_index = guard.next();
                let next = if next_index > 0 {
                    self.index(next_index as usize)
                } else {
                    None
                };

                if let Some(next) = next {
                    let mut expected_next = next;
                    #[allow(clippy::option_if_let_else)]
                    if let Some(next_guard) = next.lock() {
                        if !guard.allocated()
                            && !next_guard.allocated()
                            && next_guard.valid()
                            && guard
                                .low_pointer()
                                .wrapping_add(guard.allocation_length().try_into().unwrap())
                                == next_guard.low_pointer()
                            && guard.allocation_length() + next_guard.allocation_length() <= 4096
                        {
                            guard.set_allocation_length(
                                guard.allocation_length() + next_guard.allocation_length(),
                            );
                            guard.set_next(next_guard.next());
                            next_guard.set_valid(false);
                            expected_next = current;
                        }
                    } else {
                        todo!()
                    }

                    current = expected_next;
                } else {
                    break;
                }
            } else {
                return;
            }
        }
    }
}

impl Default for AllocationTable {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
pub fn test() {
    use std::boxed::Box;

    let region = Box::leak(Box::new([0u8; 4096]));
    let mut table = AllocationTable::construct_with_region(region);

    for _ in 0..10 {
        table.add_region(Box::leak(Box::new([0u8; 4096])));
    }

    // table.debug_dump_all();

    let v = table.alloc(128, 128);
    warn!("{:?}", v);
    let w = table.alloc(16, 4);
    warn!("{:?}", w);
    let u = table.alloc(16, 4);
    warn!("{:?}", u);
    // table.debug_dump_all();

    table.free(v.unwrap() as usize);
    table.free(w.unwrap() as usize);

    table.debug_dump_all();

    table.coalesce_free_regions();

    table.debug_dump_all();
    table.free(u.unwrap() as usize);
    table.coalesce_free_regions();
    table.debug_dump_all();

    panic!()
}
