use super::INodeReference;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSystemError {
    GenericError,
    BadInode(INodeReference),
    BadInodeWrongDevice(INodeReference),
    NoMountedFilesystem,
    CorruptedFilesystem,
    PathNotFound,
    NotDirectory,
}
