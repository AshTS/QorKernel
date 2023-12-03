/// Hardware Thread Identifier
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HartID(pub usize);

impl core::convert::From<usize> for HartID {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

/// Process Identifier
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcessID(pub u16);

impl core::convert::From<u16> for ProcessID {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

pub type PID = ProcessID;

/// User Identifier
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserID(pub u16);

impl core::convert::From<u16> for UserID {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

pub type UID = UserID;

/// Group Identifier
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GroupID(pub u16);

impl core::convert::From<u16> for GroupID {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

pub type GID = GroupID;

/// File Descriptor
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileDescriptor(pub u64);

impl core::convert::From<u64> for FileDescriptor {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
