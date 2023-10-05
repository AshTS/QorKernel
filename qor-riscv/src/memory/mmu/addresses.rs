/// Virtual Address
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtualAddress(pub u64);

/// Physical Address
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalAddress(pub u64);

impl VirtualAddress {
    /// Get segment 0 of the virtual address
    #[must_use]
    pub const fn vpn0(self) -> u64 {
        (self.0 >> 12) & 0b1_1111_1111
    }

    /// Get segment 1 of the virtual address
    #[must_use]
    pub const fn vpn1(self) -> u64 {
        (self.0 >> 21) & 0b1_1111_1111
    }

    /// Get segment 2 of the virtual address
    #[must_use]
    pub const fn vpn2(self) -> u64 {
        (self.0 >> 30) & 0b1_1111_1111
    }

    /// Get the indexed segment of the virtual address
    #[must_use]
    pub const fn vpn(self, index: usize) -> u64 {
        match index {
            0 => self.vpn0(),
            1 => self.vpn1(),
            2 => self.vpn2(),
            _ => 0,
        }
    }

    /// Get the page offset of the virtual address
    #[must_use]
    pub const fn page_offset(self) -> u64 {
        self.0 & 0b1111_1111_1111
    }

    /// Get the inner address of the virtual address
    #[must_use]
    pub const fn inner(self) -> u64 {
        self.0
    }
}

impl PhysicalAddress {
    /// Get segment 0 of the physical address
    #[must_use]
    pub const fn ppn0(self) -> u64 {
        (self.0 >> 12) & 0b1_1111_1111
    }

    /// Get segment 1 of the physical address
    #[must_use]
    pub const fn ppn1(self) -> u64 {
        (self.0 >> 21) & 0b1_1111_1111
    }

    /// Get segment 2 of the physical address
    #[must_use]
    pub const fn ppn2(self) -> u64 {
        (self.0 >> 30) & 0b11_1111_1111_1111_1111_1111_1111
    }

    /// Get the indexed segment of the physical address
    #[must_use]
    pub const fn ppn(self, index: usize) -> u64 {
        match index {
            0 => self.ppn0(),
            1 => self.ppn1(),
            2 => self.ppn2(),
            _ => 0,
        }
    }

    /// Get the page offset of the physical address
    #[must_use]
    pub const fn page_offset(self) -> u64 {
        self.0 & 0b1111_1111_1111
    }

    /// Get the inner address of the physical address
    #[must_use]
    pub const fn inner(self) -> u64 {
        self.0
    }
}

impl<T> core::convert::From<*mut T> for PhysicalAddress {
    fn from(value: *mut T) -> Self {
        Self(value as u64)
    }
}
