use crate::structures::mem::{PermissionFlags, PermissionFlag};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramHeaderFlag {
    Execute,
    Write,
    Read,
}

impl ProgramHeaderFlag {
    #[must_use]
    pub const fn repr(&self) -> u32 {
        match self {
            Self::Execute => 0x1,
            Self::Write => 0x2,
            Self::Read => 0x4,
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ProgramHeaderFlags {
    data: u32,
}

impl ProgramHeaderFlags {
    #[must_use]
    pub const fn new(data: u32) -> Self {
        Self { data }
    }

    #[must_use]
    pub const fn data(&self) -> u32 {
        self.data
    }

    #[must_use]
    pub const fn flag(&self, flag: ProgramHeaderFlag) -> bool {
        self.data & flag.repr() != 0
    }

    pub fn set_flag_state(&mut self, flag: ProgramHeaderFlag, state: bool) {
        if state {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }

    pub fn set_flag(&mut self, flag: ProgramHeaderFlag) {
        self.data |= flag.repr();
    }

    pub fn clear_flag(&mut self, flag: ProgramHeaderFlag) {
        self.data &= !flag.repr();
    }
}

impl core::ops::BitOr<ProgramHeaderFlag> for ProgramHeaderFlags {
    type Output = Self;

    fn bitor(mut self, rhs: ProgramHeaderFlag) -> Self::Output {
        self.set_flag(rhs);
        self
    }
}

impl core::ops::BitAnd<ProgramHeaderFlag> for ProgramHeaderFlags {
    type Output = bool;

    fn bitand(self, rhs: ProgramHeaderFlag) -> Self::Output {
        self.flag(rhs)
    }
}

impl core::convert::From<ProgramHeaderFlags> for PermissionFlags {
    fn from(value: ProgramHeaderFlags) -> Self {
        let mut flags = Self::new(0);

        if value.flag(ProgramHeaderFlag::Read) {
            flags = flags | PermissionFlag::Read;
        }
        if value.flag(ProgramHeaderFlag::Write) {
            flags = flags | PermissionFlag::Write;
        }
        if value.flag(ProgramHeaderFlag::Execute) {
            flags = flags | PermissionFlag::Execute;
        }

        flags
    }
}

impl core::fmt::Debug for ProgramHeaderFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ProgramHeaderFlags")
            .field("execute", &self.flag(ProgramHeaderFlag::Execute))
            .field("write", &self.flag(ProgramHeaderFlag::Write))
            .field("read", &self.flag(ProgramHeaderFlag::Read))
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionHeaderFlag {
    Write,
    Alloc,
    ExecInstr,
    Merge,
    Strings,
    InfoLink,
    LinkOrder,
    OSNonConforming,
    Group,
    TLS,
    Other(u64),
}

impl SectionHeaderFlag {
    #[must_use]
    pub const fn repr(&self) -> u64 {
        match self {
            Self::Write => 0x1,
            Self::Alloc => 0x2,
            Self::ExecInstr => 0x4,
            Self::Merge => 0x10,
            Self::Strings => 0x20,
            Self::InfoLink => 0x40,
            Self::LinkOrder => 0x80,
            Self::OSNonConforming => 0x100,
            Self::Group => 0x200,
            Self::TLS => 0x400,
            Self::Other(value) => *value,
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SectionHeaderFlags {
    data: u64,
}

impl SectionHeaderFlags {
    #[must_use]
    pub const fn new(data: u64) -> Self {
        Self { data }
    }

    #[must_use]
    pub const fn data(&self) -> u64 {
        self.data
    }

    #[must_use]
    pub const fn flag(&self, flag: SectionHeaderFlag) -> bool {
        self.data & flag.repr() != 0
    }

    pub fn set_flag_state(&mut self, flag: SectionHeaderFlag, state: bool) {
        if state {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }

    pub fn set_flag(&mut self, flag: SectionHeaderFlag) {
        self.data |= flag.repr();
    }

    pub fn clear_flag(&mut self, flag: SectionHeaderFlag) {
        self.data &= !flag.repr();
    }
}
