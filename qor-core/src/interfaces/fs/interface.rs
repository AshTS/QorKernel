use super::{DirectoryEntry, FileDescriptor, FileSystemError, INodeData, INodeReference};

use alloc::{boxed::Box, vec::Vec};

#[async_trait::async_trait]
pub trait FileSystem {
    async fn root_inode(&self) -> Result<INodeReference, FileSystemError>;
    async fn inode_data(&self, inode: INodeReference) -> Result<INodeData, FileSystemError>;
    async fn directory_entries(
        &self,
        inode: INodeReference,
    ) -> Result<Vec<DirectoryEntry<'_>>, FileSystemError>;
    async fn open(&self, inode: INodeReference)
        -> Result<Box<dyn FileDescriptor>, FileSystemError>;
}

pub trait MountableFileSystem: FileSystem {
    fn set_mount_device_id(&self, device_id: usize);
}

pub trait MountingFilesystem: FileSystem {
    /// Mount a filesystem at a given inode.
    fn mount_filesystem(
        &mut self,
        inode: INodeReference,
        device: alloc::sync::Arc<dyn MountableFileSystem + Send + Sync + 'static>,
    );
}
