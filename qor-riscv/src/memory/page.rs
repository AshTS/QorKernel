pub const PAGE_SIZE: usize = 4096;

#[repr(C)]
#[repr(align(4096))]
pub struct Page([u8; PAGE_SIZE]);

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
