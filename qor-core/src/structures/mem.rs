#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionFlag {
    Execute,
    Write,
    Read,
}

impl PermissionFlag {
    #[must_use]
    pub const fn repr(&self) -> u32 {
        match self {
            Self::Execute => 0x1,
            Self::Write => 0x2,
            Self::Read => 0x4,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PermissionFlags {
    data: u32,
}

impl PermissionFlags {
    #[must_use]
    pub const fn new(data: u32) -> Self {
        Self { data }
    }

    #[must_use]
    pub const fn data(&self) -> u32 {
        self.data
    }

    #[must_use]
    pub const fn flag(&self, flag: PermissionFlag) -> bool {
        self.data & flag.repr() != 0
    }

    pub fn set_flag_state(&mut self, flag: PermissionFlag, state: bool) {
        if state {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }

    pub fn set_flag(&mut self, flag: PermissionFlag) {
        self.data |= flag.repr();
    }

    pub fn clear_flag(&mut self, flag: PermissionFlag) {
        self.data &= !flag.repr();
    }
}

impl core::ops::BitOr<PermissionFlag> for PermissionFlags {
    type Output = Self;

    fn bitor(mut self, rhs: PermissionFlag) -> Self::Output {
        self.set_flag(rhs);
        self
    }
}

impl core::ops::BitAnd<PermissionFlag> for PermissionFlags {
    type Output = bool;

    fn bitand(self, rhs: PermissionFlag) -> Self::Output {
        self.flag(rhs)
    }
}