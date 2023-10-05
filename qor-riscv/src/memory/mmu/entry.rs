use super::addresses::PhysicalAddress;

const PPN2_MASK: u64 = (1 << 26) - 1;
const PPN2_BIT: u32 = 28;

const PPN1_MASK: u64 = (1 << 9) - 1;
const PPN1_BIT: u32 = 19;

const PPN0_MASK: u64 = (1 << 9) - 1;
const PPN0_BIT: u32 = 10;

const RSW_MASK: u64 = 0b11;
const RSW_BIT: u32 = 8;

const DIRTY_BIT: u64 = 1 << 7;
const ACCESSED_BIT: u64 = 1 << 6;
const GLOBAL_BIT: u64 = 1 << 5;
const USER_BIT: u64 = 1 << 4;
const EXECUTE_BIT: u64 = 1 << 3;
const WRITE_BIT: u64 = 1 << 2;
const READ_BIT: u64 = 1 << 1;
const VALID_BIT: u64 = 1 << 0;

/// Wrapper for entries in a page table.
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)]
pub struct PageTableEntry(u64);

/// Flags gating the permission of a mapping
#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)]
pub enum EntryPermissionFlags {
    None = 0,
    ReadOnly = 2,
    ReadWrite = 6,
    ExecuteOnly = 8,
    ReadExecute = 10,
    ReadWriteExecute = 14,
}

/// Flags for modifying settings of a mapping
#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalUserFlags {
    None = 0,
    Global = 32,
    User = 16,
    GlobalUser = 48,
}

impl PageTableEntry {
    /// Construct an invalid [`PageTableEntry`]
    #[must_use]
    pub const fn invalid_entry() -> Self {
        Self(0)
    }

    /// Construct a valid [`PageTableEntry`]
    #[must_use]
    pub const fn construct_valid(
        ppn: [u64; 3],
        rsw: u64,
        gu_frames: GlobalUserFlags,
        perm_flags: EntryPermissionFlags,
    ) -> Self {
        Self(
            ((ppn[2] & PPN2_MASK) << PPN2_BIT)
                | ((ppn[1] & PPN1_MASK) << PPN1_BIT)
                | ((ppn[0] & PPN0_MASK) << PPN0_BIT)
                | ((rsw & RSW_MASK) << RSW_BIT)
                | gu_frames as u64
                | perm_flags as u64
                | VALID_BIT,
        )
    }

    /// Return a boolean value representing the valid flag
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.0 & VALID_BIT > 0
    }

    /// Return a boolean value representing the global bit
    #[must_use]
    pub const fn global(self) -> bool {
        self.0 & GLOBAL_BIT > 0
    }

    /// Return a boolean value representing the user bit
    #[must_use]
    pub const fn user(self) -> bool {
        self.0 & USER_BIT > 0
    }

    /// Return a boolean value representing the accessed bit
    #[must_use]
    pub const fn accessed(self) -> bool {
        self.0 & ACCESSED_BIT > 0
    }

    /// Return a boolean value representing the dirty bit
    #[must_use]
    pub const fn dirty(self) -> bool {
        self.0 & DIRTY_BIT > 0
    }

    /// Return a boolean value representing the read bit
    #[must_use]
    pub const fn read(self) -> bool {
        self.0 & READ_BIT > 0
    }

    /// Return a boolean value representing the write bit
    #[must_use]
    pub const fn write(self) -> bool {
        self.0 & WRITE_BIT > 0
    }

    /// Return a boolean value representing the execute bit
    #[must_use]
    pub const fn execute(self) -> bool {
        self.0 & EXECUTE_BIT > 0
    }

    /// Extract the physical page number segment 0
    #[must_use]
    pub const fn ppn0(self) -> u64 {
        PPN0_MASK & (self.0 >> PPN0_BIT)
    }

    /// Set the physical page number segment 0
    pub fn set_ppn0(&mut self, value: u64) {
        self.0 &= !(PPN0_MASK << PPN0_BIT);
        self.0 |= (value & PPN0_MASK) << PPN0_BIT;
    }

    /// Extract the physical page number segment 1
    #[must_use]
    pub const fn ppn1(self) -> u64 {
        PPN1_MASK & (self.0 >> PPN1_BIT)
    }

    /// Set the physical page number segment 1
    pub fn set_ppn1(&mut self, value: u64) {
        self.0 &= !(PPN1_MASK << PPN1_BIT);
        self.0 |= (value & PPN1_MASK) << PPN1_BIT;
    }

    /// Extract the physical page number segment 2
    #[must_use]
    pub const fn ppn2(self) -> u64 {
        PPN2_MASK & (self.0 >> PPN2_BIT)
    }

    /// Set the physical page number segment 2
    pub fn set_ppn2(&mut self, value: u64) {
        self.0 &= !(PPN2_MASK << PPN2_BIT);
        self.0 |= (value & PPN2_MASK) << PPN2_BIT;
    }

    /// Get the indexed segment of the physical page number
    #[must_use]
    pub const fn ppn(self, index: usize) -> u64 {
        match index {
            0 => self.ppn0(),
            1 => self.ppn1(),
            2 => self.ppn2(),
            _ => 0,
        }
    }

    /// Get the full physical address from the [`PageTableEntry`]
    #[must_use]
    pub const fn physical_address(self) -> PhysicalAddress {
        let data =
            (self.ppn2() << PPN2_BIT) | (self.ppn1() << PPN1_BIT) | (self.ppn0() << PPN0_BIT);
        PhysicalAddress(data << 2)
    }

    /// Get the physical address with the given level
    #[must_use]
    pub const fn ppn_level(self, level: usize) -> PhysicalAddress {
        let mask = (1 << (12 + level * 9)) - 1;
        PhysicalAddress((self.0 << 2) & !mask)
    }

    /// Extract the global user flags
    #[must_use]
    pub const fn global_user_flags(self) -> Option<GlobalUserFlags> {
        match self.0 & (GLOBAL_BIT | USER_BIT) {
            0 => Some(GlobalUserFlags::None),
            32 => Some(GlobalUserFlags::Global),
            16 => Some(GlobalUserFlags::User),
            48 => Some(GlobalUserFlags::GlobalUser),
            _ => None,
        }
    }

    ///  Set the global user flags
    pub fn set_global_user_flags(&mut self, flags: GlobalUserFlags) {
        self.0 &= !(GLOBAL_BIT | USER_BIT);
        self.0 |= flags as u64;
    }

    /// Extract the permission flags
    #[must_use]
    pub const fn permission_flags(self) -> Option<EntryPermissionFlags> {
        match self.0 & (READ_BIT | WRITE_BIT | EXECUTE_BIT) {
            0 => Some(EntryPermissionFlags::None),
            2 => Some(EntryPermissionFlags::ReadOnly),
            6 => Some(EntryPermissionFlags::ReadWrite),
            8 => Some(EntryPermissionFlags::ExecuteOnly),
            10 => Some(EntryPermissionFlags::ReadExecute),
            14 => Some(EntryPermissionFlags::ReadWriteExecute),
            _ => None,
        }
    }

    ///  Set the permission flags
    pub fn set_permission_flags(&mut self, flags: EntryPermissionFlags) {
        self.0 &= !(READ_BIT | WRITE_BIT | EXECUTE_BIT);
        self.0 |= flags as u64;
    }

    /// Return a boolean, true if the entry is a leaf
    #[must_use]
    pub const fn is_leaf(self) -> bool {
        self.execute() | self.read() | self.write()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct FlagDebug(PageTableEntry);

impl core::fmt::Debug for FlagDebug {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "\"{}{}{}{}{}{}{}{}\"",
            if self.0.dirty() { "d" } else { " " },
            if self.0.accessed() { "a" } else { " " },
            if self.0.global() { "g" } else { " " },
            if self.0.user() { "u" } else { " " },
            if self.0.execute() { "x" } else { " " },
            if self.0.write() { "w" } else { " " },
            if self.0.read() { "r" } else { " " },
            if self.0.is_valid() { "v" } else { " " }
        )
    }
}

impl core::fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PageTableEntry")
            .field("PPN2", &self.ppn2())
            .field("PPN1", &self.ppn1())
            .field("PPN0", &self.ppn0())
            .field("Flags", &FlagDebug(*self))
            .finish()?;

        Ok(())
    }
}

impl core::fmt::Display for GlobalUserFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::None => write!(f, "--"),
            Self::Global => write!(f, "g-"),
            Self::User => write!(f, "-u"),
            Self::GlobalUser => write!(f, "gu"),
        }
    }
}

impl core::fmt::Display for EntryPermissionFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::None => write!(f, "---"),
            Self::ReadOnly => write!(f, "r--"),
            Self::ReadWrite => write!(f, "rw-"),
            Self::ExecuteOnly => write!(f, "--x"),
            Self::ReadExecute => write!(f, "r-x"),
            Self::ReadWriteExecute => write!(f, "rwx"),
        }
    }
}
