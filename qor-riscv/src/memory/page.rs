use qor_core::memory::MemoryUnit;

pub const PAGE_SIZE_U32: u32 = 4096;
pub const PAGE_SIZE: usize = PAGE_SIZE_U32 as usize;

#[repr(C)]
#[repr(align(4096))]
pub struct Page([u8; PAGE_SIZE]);

#[allow(clippy::module_name_repetitions)]
pub type PageCount = MemoryUnit<{ PAGE_SIZE }>;

impl Page {
    /// Creates a new [`Page`] which is zero allocated.
    #[must_use]
    pub const fn new() -> Self {
        Self([0; PAGE_SIZE])
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::new()
    }
}
