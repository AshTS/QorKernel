use alloc::borrow::Cow;

use crate::structures::{
    id::{GroupID, UserID},
    time::UnixTimestamp,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct INodeReference {
    pub inode: usize,
    pub device: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileMode(u16);

impl core::convert::From<u16> for FileMode {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl core::convert::From<FileMode> for u16 {
    fn from(value: FileMode) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct INodeData {
    pub mode: FileMode,
    pub link_count: usize,
    pub uid: UserID,
    pub gid: GroupID,
    pub size: usize,
    pub access_time: UnixTimestamp,
    pub modify_time: UnixTimestamp,
    pub change_time: UnixTimestamp,
    pub reference: INodeReference,
}

impl INodeData {
    #[must_use]
    pub const fn is_directory(&self) -> bool {
        self.mode.0 & 0x4000 > 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryEntry<'a> {
    pub inode: INodeReference,
    pub name: Cow<'a, str>,
}
