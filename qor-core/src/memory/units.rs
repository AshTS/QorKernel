/// Memory Unit Generic Type, stores the size of a memory region as a number of a particular size of units
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryUnit<const SCALE: usize>(usize);

/// Memory Unit for Mebibytes
pub type MiByteCount = MemoryUnit<{ 1024 * 1024 }>;

/// Memory Unit for Kibibytes
pub type KiByteCount = MemoryUnit<1024>;

/// Memory Unit for Bytes
pub type ByteCount = MemoryUnit<1>;

impl<const SCALE: usize> MemoryUnit<SCALE> {
    /// Construct the type from a particular number of units
    #[must_use]
    pub const fn new(units: usize) -> Self {
        Self(units)
    }

    /// Get the raw number of units
    #[must_use]
    pub const fn raw(&self) -> usize {
        self.0
    }

    /// Get the raw number of bytes
    #[must_use]
    pub const fn raw_bytes(&self) -> usize {
        SCALE * self.0
    }

    /// Get a mutable reference to the number of bytes
    pub fn mut_raw(&mut self) -> &mut usize {
        &mut self.0
    }
}

impl<const SRC: usize> MemoryUnit<SRC> {
    #[must_use]
    pub const fn convert<const DEST: usize>(&self) -> MemoryUnit<DEST> {
        MemoryUnit((self.raw_bytes() + DEST - 1) / DEST)
    }
}

impl core::fmt::Display for MemoryUnit<1> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} B", self.0)
    }
}

impl core::fmt::Display for MemoryUnit<1024> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} KiB", self.0)
    }
}

impl core::fmt::Display for MemoryUnit<{ 1024 * 1024 }> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} MiB", self.0)
    }
}
