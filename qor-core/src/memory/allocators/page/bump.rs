use core::{
    mem::{align_of, size_of},
    ops::Range,
};

/// Bump allocator for permanently allocating memory in chunks of pages.
///
/// If `Page` is a zero sized type, some pointer arithmetic is no longer valid and many functions of this allocator
/// will panic as a result.
///
#[repr(align(16))]
pub struct PageBumpAllocator<Page> {
    // Safety Requirements:
    // - If the value of `walking_pointer` is less than that of `end_pointer`
    // and `end_pointer` is non-null, every page at `walking_pointer`,
    // `walking_pointer.add(1)`, `walking_pointer.add(2)`, and so on up to and
    // not including `end_pointer` is aligned and points to an unallocated page.
    //
    // - If the value in `end_pointer` is null, then no guarantee is made about
    // the value in `walking_pointer`, if `end_pointer` is not null, it is
    // properly aligned and points to the end of a range of pages which are
    // able to be allocated by the allocator.
    walking_pointer: core::sync::atomic::AtomicPtr<Page>,
    end_pointer: core::sync::atomic::AtomicPtr<Page>,
    total_pages: core::sync::atomic::AtomicUsize,
}

/// Errors possible to be returned by the allocator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationError {
    OutOfMemory {
        remaining: usize,
        total: usize,
        requested: usize,
    },
    Uninitialized,
}

impl<Page: 'static> PageBumpAllocator<Page> {
    /// Construct an empty `PageBumpAllocator` this allocator will always return a `AllocationError`
    #[must_use]
    pub const fn new() -> Self {
        // Construction Checks:
        // `end_pointer` is null, so no guarantee is made about the value in `walking_pointer`.
        Self {
            walking_pointer: core::sync::atomic::AtomicPtr::new(core::ptr::null_mut()),
            end_pointer: core::sync::atomic::AtomicPtr::new(core::ptr::null_mut()),
            total_pages: core::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Update the allocator to point to a range of pages
    ///
    /// # Safety
    /// - The provided `region` must be a non-empty range of pointers where both the start and end are properly aligned,
    /// and neither are null.
    /// - The end of `region` must be greater than the start of `region`.
    /// - Starting at the first pointer in `region`, all pointers up to but not including the end of `region` are valid
    /// pages which are available to be allocated by the allocator.
    /// - Every `Page` pointed to within `region` must be a valid value of `Page` (note that the type given as a `Page`
    /// should not have any invalid representations).
    /// - The distance between the start and end of `region` must not exceed `isize::MAX` bytes.
    ///
    /// # Panics
    ///
    /// This function will panic if `Page` is a Zero Sized type.
    pub unsafe fn assign_region(&self, region: Range<*mut Page>) {
        // We start by updating the `walking_pointer`. Because the ordering on the subsequent store to `end_pointer` is
        // `Release`, this store will be executed before the `end_pointer` is loaded using `Acquire` ordering.
        self.walking_pointer
            .store(region.start, core::sync::atomic::Ordering::Relaxed);

        // - The safety requirements of `assign_region` state that the end of `region` must be greater than the start of `region`, so the
        // difference must be nonnegative.
        // - Both the start and end of `region` point to either part of the region of memory being assigned to the allocator, or directly after it.
        // - Both the start and end originate from the range of memory being assigned.
        // - Both the start and end are required to be properly aligned, thus the difference must be a multiple of that of `Page`
        // - The distance between the start and end of `region` are required to not exceed `isize::MAX` bytes.
        // - The greater than requirement prevents wrapping around the address space.
        let page_count = unsafe { region.end.sub_ptr(region.start) };

        // There are no restrictions on the value for `total_pages` as it is only used for debugging.
        self.total_pages
            .store(page_count, core::sync::atomic::Ordering::Relaxed);

        // By the safety requirements of `assign_region`, `region.end` must not be null, thus we need to check the other constraints.
        // The safety requirements give that `region.start` must be less than `region.end`, thus, every page pointed to must be aligned
        // and a valid value of `Page` which is free to be allocated. This is also given by the safety requirements of `assign_region`
        self.end_pointer
            .store(region.end, core::sync::atomic::Ordering::Release);

        trace!(
            "Initializing bump allocator with range {:x?} to {:x?}",
            region.start,
            region.end
        );
    }

    /// This function attempts to allocate `page_count` pages of memory from the allocator.
    ///
    /// # Errors
    ///
    /// If the allocator has not been initialized an `AllocationError::Uninitialized` error is returned. If the
    /// allocator has been initialized, but does not have enough memory to complete the allocation, an
    /// `AllocationError::OutOfMemory` error is returned.
    ///
    /// # Panics
    ///
    /// This function will panic if `Page` is a Zero Sized type.
    pub fn allocate(&self, page_count: usize) -> Result<&'static mut [Page], AllocationError> {
        // This read using `Acquire` ordering means any modifications made in the `assign_region` function have already taken place.
        let end_pointer = self.end_pointer.load(core::sync::atomic::Ordering::Acquire);

        if end_pointer.is_null() {
            Err(AllocationError::Uninitialized)
        } else {
            // Once control makes it here, `end_pointer` must not be null
            let walking_pointer = self
                .walking_pointer
                .fetch_ptr_add(page_count, core::sync::atomic::Ordering::Relaxed);

            // If `walking_pointer` is greater than or equal to `end_pointer`, there is a memory exhaustion error.
            if walking_pointer >= end_pointer {
                Err(AllocationError::OutOfMemory {
                    remaining: 0,
                    total: self.total_pages.load(core::sync::atomic::Ordering::Acquire),
                    requested: page_count,
                })
            } else {
                // Once control makes it here, `walking_pointer` is known to be less than
                // `end_pointer`, thus every page at `walking_pointer`,
                // `walking_pointer.add(1)`, `walking_pointer.add(2)`, and so on up to and
                // not including `end_pointer` is aligned and points to an unallocated page.

                // Compute the pages remaining free in the allocator
                // Safety:
                // - The distance between pointers is known to be non-negative as `walking_pointer < end_pointer`.
                // - `end_pointer` is non null and thus known to be properly aligned and pointing to just after the assigned range.
                // - `walking_pointer` is known to be valid and part of the assigned range.
                // - Both are known to be properly aligned, and thus the difference will be a multiple of the size of `Page`.
                // - The only way to initialize `walking_pointer` was to have it have a distance of less than `isize::MAX` bytes
                // from `end_pointer`, `walking_pointer` only increases, and is less than `end_pointer`, and is thus within that distance.
                // - A similar argument prevents wrapping.
                let free_pages = unsafe { end_pointer.sub_ptr(walking_pointer) };

                if free_pages >= page_count {
                    // Safety:
                    // - At least `page_count` pages remain between `walking_pointer` and `end_pointer`, and all pages between the two are known to be valid for being allocated.
                    // - All pages in that range are known to be valid representations of `Page`.
                    // - This memory will never again be accessed, because `page_count` was added to `walking_pointer`, and it is thus out of this range. Because `walking_pointer`
                    // is never decreased, it can not return to within this range. Thus exclusive access is achieved.
                    // - Because the distance between `walking_pointer` and `end_pointer` was known to be less than `isize::MAX` bytes, then this allocation, which is known to be
                    // less than or equal to that size must also be less than `isize::MAX` bytes.
                    Ok(unsafe { core::slice::from_raw_parts_mut(walking_pointer, page_count) })
                } else {
                    Err(AllocationError::OutOfMemory {
                        remaining: free_pages,
                        total: self.total_pages.load(core::sync::atomic::Ordering::Acquire),
                        requested: page_count,
                    })
                }
            }
        }
    }

    /// Allocate space for a particular value from the allocator, returning a static mutable reference to it.
    ///
    /// # Errors
    ///
    /// If the allocator has not been initialized an `AllocationError::Uninitialized` error is returned. If the
    /// allocator has been initialized, but does not have enough memory to complete the allocation, an
    /// `AllocationError::OutOfMemory` error is returned.
    ///
    /// # Panics
    ///
    /// This function will panic if `Page` is a Zero Sized type. Additionally, this function will panic if `T` has a
    /// greater alignment requirement than `Page`.
    pub fn allocate_object<T: Sized>(&self, object: T) -> Result<&'static mut T, AllocationError> {
        // Compute the number of pages required
        let page_size = size_of::<Page>();
        assert!(page_size > 0);
        let object_size = size_of::<T>();
        let pages_required = (object_size + page_size - 1) / page_size;

        // Allocate the necessary memory
        let allocated = self.allocate(pages_required)?;
        let allocated_ptr = allocated.as_mut_ptr().cast::<T>();

        // Verify alignment of `T` is not greater than that of `Page`.
        assert!(align_of::<T>() <= align_of::<Page>());

        // Safety:
        // - The above assertion ensures that the pointer is properly aligned.
        // - `allocated_ptr` came from `slice::as_mut_ptr()` and thus must be
        //   valid for writes.
        unsafe { allocated_ptr.write(object) };

        // Safety:
        // - The above assertion ensures that the pointer is properly aligned.
        // - The memory came from the single allocation performed to construct
        //   `allocated`.
        // - `allocated_ptr` points to the valid construction of `T` which was
        //   written from `object`.
        // - The resulting lifetime is static because this is the bump
        //   allocator, and we got the static `Page` range allocation.
        Ok(unsafe { allocated_ptr.as_mut() }.unwrap())
    }
}

impl<Page: 'static> core::default::Default for PageBumpAllocator<Page> {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(feature = "std")]
#[cfg(test)]
mod test {
    #[test]
    pub fn test_allocator() {
        let mem = std::boxed::Box::leak(std::boxed::Box::new([0usize; 1024]));
        let allocator = super::PageBumpAllocator::new();
        unsafe {
            allocator.assign_region(mem.as_mut_ptr_range());
        }
        allocator.allocate(54).unwrap();
    }
}
