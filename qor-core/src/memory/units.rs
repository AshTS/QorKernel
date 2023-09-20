const KIB: usize = 1024;
const MIB: usize = 1024 * 1024;
const GIB: usize = 1024 * 1024 * 1024;

/// Memory Unit Generic Type, stores the size of a memory region as a number of a particular size of units
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryUnit<const SCALE: usize>(usize);

/// Memory Unit for Mebibytes
pub type MiByteCount = MemoryUnit<MIB>;

/// Memory Unit for Kibibytes
pub type KiByteCount = MemoryUnit<KIB>;

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

impl<const SIZE: usize> core::fmt::Display for MemoryUnit<SIZE> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let raw_bytes = self.raw_bytes();

        if raw_bytes >= GIB && raw_bytes % GIB == 0 {
            write!(f, "{} GiB", raw_bytes / GIB)
        } else if raw_bytes >= MIB && raw_bytes % MIB == 0 {
            write!(f, "{} MiB", raw_bytes / MIB)
        } else if raw_bytes >= KIB && raw_bytes % KIB == 0 {
            write!(f, "{} KiB", raw_bytes / KIB)
        } else {
            write!(f, "{raw_bytes} B")
        }
    }
}
