use super::{DirectoryEntry, FileDescriptor, FileSystemError, INodeData, INodeReference};

use alloc::{boxed::Box, vec::Vec, sync::Arc};

#[async_trait::async_trait]
pub trait FileSystem {
    async fn root_inode(&self) -> Result<INodeReference, FileSystemError>;
    async fn inode_data(&self, inode: INodeReference) -> Result<INodeData, FileSystemError>;
    async fn directory_entries(
        &self,
        inode: INodeReference,
    ) -> Result<Vec<DirectoryEntry<'_>>, FileSystemError>;
    async fn open(&self, inode: INodeReference)
        -> Result<Arc<dyn FileDescriptor>, FileSystemError>;
    async fn read_to_data(&self, inode: INodeReference) -> Result<Vec<u8>, FileSystemError>;
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

#[async_trait::async_trait]
pub trait PathLookup {
    async fn lookup(&self, path: &str) -> Result<INodeReference, FileSystemError>;
    async fn reverse_lookup(
        &self,
        inode: INodeReference,
    ) -> Result<Option<alloc::string::String>, FileSystemError>;
    async fn invalidate_cache(&self, inode: INodeReference) -> Result<(), FileSystemError>;
    async fn walk_children(&self, inode: INodeReference) -> Result<usize, FileSystemError>;
}

pub trait ParentFileSystem: MountingFilesystem + PathLookup {}
