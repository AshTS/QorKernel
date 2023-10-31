use core::mem::{align_of, size_of};

use crate::utils::bitmap::{BitmapError, BitmapLock};

/// Page grained bitmap allocator
pub struct PageBitmapAllocator<Page> {
    // Safety Requirements:
    // - If the value of `start_pointer` is non-null, then it must point to a
    //   region of memory containing `bitmap.length` `Page`s, and
    //    - If the corresponding bit in the `bitmap` is cleared, then the `Page`
    //      at that index is free to be allocated.
    bitmap: BitmapLock,
    start_pointer: core::sync::atomic::AtomicPtr<Page>,
}

/// Errors possible to be returned by the allocator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationError {
    OutOfMemory { requested: usize },
    Uninitialized,
}

/// Bitmap allocated smart pointer
pub struct PageBox<'a, Page: 'static, T> {
    // Safety Requirements:
    // - The allocator pointed to by `allocator` has made an allocation at `ptr` with a length of `page_count` pages.
    // - `ptr` is valid, properly aligned, and uniquely owned, and thus dereferenceable.
    // - This allocation lives exactly as long as this `PageBox`, as the allocation can only safely be freed when it is
    //   dropped.
    allocator: &'a PageBitmapAllocator<Page>,
    ptr: core::ptr::NonNull<T>,
    page_count: usize,
}

impl<'a, Page: 'static, T> PageBox<'a, Page, T> {
    /// Get the wrapped pointer as a pointer
    #[must_use]
    pub const fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }
}

impl<'a, Page: 'static, T> core::ops::Deref for PageBox<'a, Page, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // Safety:
        // - `ptr` is properly aligned and dereferenceable.
        // - `ptr` points to a valid instance of `T`
        // - The allocation must live as long as the `PageBox`, and since the lifetime of the resulting reference is
        //   that of the borrow of `self`, it must be valid for that lifetime.
        unsafe { self.ptr.as_ref() }
    }
}

impl<'a, Page: 'static, T> core::ops::DerefMut for PageBox<'a, Page, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety:
        // - `ptr` is properly aligned and dereferenceable.
        // - `ptr` points to a valid instance of `T`
        // - The allocation must live as long as the `PageBox`, and since the lifetime of the resulting reference is
        //   that of the borrow of `self`, it must be valid for that lifetime.
        unsafe { self.ptr.as_mut() }
    }
}

impl<'a, Page: 'static, T> core::ops::Drop for PageBox<'a, Page, T> {
    fn drop(&mut self) {
        // Safety:
        // - `ptr` is guaranteed to be valid and properly aligned, and refer to an allocation of `self.page_count` pages.
        unsafe {
            self.allocator
                .free(self.ptr.as_ptr().cast::<Page>(), self.page_count)
                .unwrap();
        }
    }
}

impl<Page> PageBitmapAllocator<Page> {
    /// Creates a new [`PageBitmapAllocator<Page>`] with an empty allocation space.
    #[must_use]
    pub const fn new() -> Self {
        // Safety Requirements:
        // The value of `start_pointer` is null, and thus we have no special requirements.
        Self {
            bitmap: BitmapLock::new(),
            start_pointer: core::sync::atomic::AtomicPtr::new(core::ptr::null_mut()),
        }
    }

    /// Construct a [`PageBitmapAllocator<Page>`] to refer to a slice of pages
    ///
    /// # Panics
    ///
    /// This function will panic if `Page` is not aligned to 8 byte boundaries or more.
    pub fn from_pages(data: &'static mut [Page]) -> Self {
        // Immediately assert that `Page` is properly aligned (that is, its alignment is greater than or equal to that of `AtomicU64`).
        assert!(align_of::<Page>() >= align_of::<core::sync::atomic::AtomicU64>());

        // Calculate the number of `u64`s that can be put into a `Page`
        let u64s_per_page = size_of::<Page>() / 8;
        let denominator = 64 * u64s_per_page + 1;
        let pages_for_bitmap = (data.len() + denominator - 1) / denominator;

        // Split the available data into the space for the bitmap, and the space for the allocations
        let (for_bitmap, for_allocation) = data.split_at_mut(pages_for_bitmap);

        // Convert the bitmap slice into a slice of atomic `u64`s

        // First, take the pointer to the bitmap slice
        let bitmap_page_count = for_bitmap.len();
        let bitmap_pointer_as_pages = for_bitmap.as_mut_ptr();
        let bitmap_pointer_as_au64 =
            bitmap_pointer_as_pages.cast::<core::sync::atomic::AtomicU64>();

        info!(
            "{} pages for bitmap at {:x?}",
            bitmap_page_count, bitmap_pointer_as_au64
        );

        // Safety:
        // - `for_bitmap` previously contained `bitmap_page_count` `Page`s, and each `Page` can store at least `u64s_per_page` `u64`s.
        //   Thus, `for_bitmap` points to at least `bitmap_page_count * u64s_per_page` `AtomicU64`s.
        // - This is a single allocated object as we have derived it from a static mutable slice.
        // - Because `bitmap_pointer_as_au64` came from `slice::as_mut_ptr()` we know it must be non-zero.
        // - `bitmap_pointer_as_au64` must be aligned because it came from a `&mut [Page]`'s pointer. Thus, it must be properly aligned
        //   for a `Page`, and this function begins with an assertion that `Page` has a greater or equal alignment as `AtomicU64`.
        // - `AtomicU64` has the same memory layout as `u64` (https://doc.rust-lang.org/core/sync/atomic/struct.AtomicU64.html), and `u64`
        //   is valid for any 8 byte sequence. Thus, no matter what the contents of the array previously, because it had to be initialized
        //   to be a valid slice of `Page`s, it must be a valid slice of `AtomicU64`s.
        // - The byte length in memory of the resulting slice is less than or equal to the byte length in memory of the previous slice,
        //   which was required to be less than `isize::MAX`.
        // - This memory came from a slice which is now consumed, and will not be used elsewhere, and thus is referenced nowhere else.
        let bitmap_slice = unsafe {
            core::slice::from_raw_parts_mut(
                bitmap_pointer_as_au64,
                bitmap_page_count * u64s_per_page,
            )
        };

        // We zero `bitmap_slice` to denote all of the pages being free.
        for entry in bitmap_slice.iter_mut() {
            *entry = core::sync::atomic::AtomicU64::new(0);
        }

        // Construct the bitmap lock
        // Note that here, we initialize `bitmap` to have a length of `for_allocation`.
        let bitmap = BitmapLock::from_data(bitmap_slice, for_allocation.len());

        // Construct the pointer pointing to the beginning of the allocation memory.
        let start_pointer = core::sync::atomic::AtomicPtr::new(for_allocation.as_mut_ptr());

        // Safety requirements:
        // `start_pointer` must not be null because it originated from `slice::as_mut_ptr()`.
        // `bitmap` was initialized to be of the length of `for_allocation`, and is all free for allocation.
        Self {
            bitmap,
            start_pointer,
        }
    }

    /// Allocate a number of pages from the [`PageBitmapAllocator<Page>`] and return a pointer to the start of that
    /// memory region.
    ///
    /// # Errors
    ///
    /// This function will return an error if the allocator is not initialized, or there is not enough memory to
    /// complete the requested allocation.
    ///
    /// # Panics
    ///
    /// This function will panic if `page_count` is set to zero.
    pub fn allocate(&self, page_count: usize) -> Result<*mut Page, AllocationError> {
        assert!(page_count > 0);

        // If the allocator is not initialized, return an error
        let start_pointer = self
            .start_pointer
            .load(core::sync::atomic::Ordering::Acquire);
        if start_pointer.is_null() {
            return Err(AllocationError::Uninitialized);
        }

        // Getting here means `start_pointer` must point to a region of memory containing `bitmap.length` `Page`s
        // Additionally, if the corresponding bit in the `bitmap` is cleared, then the `Page` at that index is free to be allocated.

        // Reserve a sequence of bits in the `bitmap`.
        match self.bitmap.reserve_sequence(page_count) {
            // Safety:
            // - `reserve_sequence` guarantees that `sequence_index` will be
            //   less than `self.bitmap.length`, which as noted above means
            //   this offset is within the allocation alloted to this allocator.
            // - The region alloted can only be constructed from a slice, which
            //   must not be greater than `isize::MAX` bytes.
            // - The sum will not wrap as the region was given as a slice, and
            //   thus is continuous.
            Ok(sequence_index) => Ok(unsafe { start_pointer.add(sequence_index) }),
            Err(BitmapError::RangeOutOfBounds { .. }) => {
                unreachable!()
            }
            Err(BitmapError::UnableToAllocate { length }) => {
                Err(AllocationError::OutOfMemory { requested: length })
            }
        }
    }

    /// Free a number of pages allocated by the [`PageBitmapAllocator<Page>`]
    ///
    /// # Errors
    ///
    /// This function will return an error if the allocator has not been initialized with memory. Note that this
    /// condition should also violate the safety contract as the allocation can not have come from an uninitialized
    /// allocator.
    ///
    /// # Safety
    ///
    /// `ptr` must be valid, and properly aligned, and point to `page_count` `Page`s of memory which were previously
    /// allocated by this allocator.
    pub unsafe fn free(&self, ptr: *mut Page, page_count: usize) -> Result<(), AllocationError> {
        let start_pointer = self
            .start_pointer
            .load(core::sync::atomic::Ordering::Acquire);
        if start_pointer.is_null() {
            return Err(AllocationError::Uninitialized);
        }

        let index = ptr.sub_ptr(start_pointer);
        self.bitmap
            .clear(index, page_count)
            .expect("Safety Contract Violated");

        Ok(())
    }

    /// Allocate a region of memory from the [`PageBitmapAllocator<Page>`] and return a `PageBox<Page, T>` to that
    /// allocation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the allocator is not initialized, or there is not enough memory to
    /// complete the requested allocation.
    ///
    /// # Panics
    ///
    /// This function will panic if `T` is zero sized.
    pub fn alloc_boxed<T: Sized>(
        &self,
        object: T,
    ) -> Result<PageBox<'_, Page, T>, AllocationError> {
        // Compute the number of pages required
        let page_size = size_of::<Page>();
        assert!(page_size > 0);
        let object_size = size_of::<T>();
        let pages_required = (object_size + page_size - 1) / page_size;

        // Allocate the necessary memory
        let allocated_ptr = self.allocate(pages_required)?.cast::<T>();

        // Verify alignment of `T` is not greater than that of `Page`.
        assert!(align_of::<T>() <= align_of::<Page>());

        // Safety:
        // - The above assertion ensures that the pointer is properly aligned.
        // - `allocated_ptr` came from `Self::allocate` and thus is valid for writes.
        unsafe { allocated_ptr.write(object) };

        Ok(PageBox {
            allocator: self,
            ptr: core::ptr::NonNull::new(allocated_ptr).unwrap(),
            page_count: pages_required,
        })
    }
}

impl<Page> Default for PageBitmapAllocator<Page> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<'a, Page, T> Sync for PageBox<'a, Page, T> where T: Sync {}

#[cfg(feature = "std")]
#[cfg(test)]
mod test {
    use std::prelude::rust_2021::*;

    use super::PageBitmapAllocator;

    #[derive(Debug, Clone, Copy)]
    #[repr(align(16))]
    struct Page([u8; 128]);

    #[test]
    pub fn simple_allocator_test() {
        let alloc_space = Box::leak(Box::new([Page([0; 128]); 1024]));
        let allocator = PageBitmapAllocator::from_pages(alloc_space);

        let mem = allocator.allocate(512).unwrap();

        assert!(allocator.allocate(512).is_err());

        unsafe { allocator.free(mem, 512) };
    }

    #[test]
    pub fn allocator_test() {
        let alloc_space = Box::leak(Box::new([Page([0; 128]); 4096]));
        let allocator = Box::leak(Box::new(PageBitmapAllocator::from_pages(alloc_space)))
            as &PageBitmapAllocator<_>;

        const THREAD_COUNT: usize = 4;
        const ALLOCATIONS: usize = 128;

        let threads = (0..THREAD_COUNT)
            .map(|i| {
                std::thread::spawn(move || {
                    for _ in 0..ALLOCATIONS {
                        let mem: *mut Page = allocator.allocate(i + 1).unwrap();

                        let _mr = unsafe { mem.as_mut().unwrap() };
                        unsafe { allocator.free(mem, i + 1) };
                    }
                })
            })
            .collect::<Vec<_>>();

        for t in threads {
            t.join().unwrap();
        }
    }

    #[test]
    pub fn alloc_box_test() {
        let alloc_space = Box::leak(Box::new([Page([0; 128]); 4096]));
        let allocator = Box::leak(Box::new(PageBitmapAllocator::from_pages(alloc_space)))
            as &PageBitmapAllocator<_>;

        let data_a = [0u64; 128];
        let data_b = [42u32; 8];

        let mut box_a = allocator.alloc_boxed(data_a).unwrap();
        let mut box_b = allocator.alloc_boxed(data_b).unwrap();

        box_a[0] = 42;
        box_b[0] = 0;

        assert_eq!(box_a[0], 42);
        assert_eq!(box_b[0], 0);

        core::mem::drop(box_a);
    }
}
