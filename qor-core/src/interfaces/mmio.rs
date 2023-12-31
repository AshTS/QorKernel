/// Interface to access memory mapped IO
#[derive(Clone, Copy)]
pub struct MMIOInterface {
    base_address: usize,
}

impl MMIOInterface {
    /// Creates a new [`MMIOInterface`] from a base address where offsets are calculated from.
    #[must_use]
    pub const fn new(base_address: usize) -> Self {
        Self { base_address }
    }

    /// Read the value at an offset within the mapped memory.
    ///
    /// # Safety
    ///
    /// The offset must be a valid offset from the base address, must be properly aligned for the type `T`, and must
    /// point to a properly initialized value of type `T`.
    #[must_use]
    pub unsafe fn read_offset<T: Copy>(&self, offset: usize) -> T {
        ((self.base_address + offset) as *mut T).read_volatile()
    }

    /// Write a value at an offset within the mapped memory.
    ///
    /// # Safety
    ///
    /// The offset must be a valid offset from the base address, and must be properly aligned for the type `T`.
    pub unsafe fn write_offset<T: Copy>(&self, offset: usize, data: T) {
        ((self.base_address + offset) as *mut T).write_volatile(data);
    }

    /// Atomic access to values at an offset within the mapped memory.
    ///
    /// # Safety
    ///
    /// The offset must be a valid offset from the base address, must be properly aligned for the type `T`.
    ///
    /// # Panics
    ///
    /// This function will panic if a null pointer is produced by the offset.
    #[must_use]
    pub unsafe fn atomic_access<T>(&self, offset: usize) -> &atomic::Atomic<T> {
        ((self.base_address + offset) as *const atomic::Atomic<T>)
            .as_ref()
            .unwrap()
    }
}
