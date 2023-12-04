use alloc::{boxed::Box, collections::BTreeMap, sync::Arc, vec::Vec};

use super::{
    DirectoryEntry, EmptyFileSystem, FileDescriptor, FileSystem, FileSystemError, INodeData,
    INodeReference, MountableFileSystem, MountingFilesystem,
};

pub struct VirtualFileSystem {
    devices: Vec<Arc<dyn MountableFileSystem + Send + Sync + 'static>>,
    mounted_filesystems: BTreeMap<INodeReference, usize>,
}

impl VirtualFileSystem {
    /// Creates a new [`VirtualFileSystem`].
    #[must_use]
    pub fn new() -> Self {
        let empty = EmptyFileSystem::new();
        empty.set_mount_device_id(1);
        Self {
            devices: alloc::vec![Arc::new(empty)],
            mounted_filesystems: BTreeMap::new(),
        }
    }

    /// Mount a filesystem at a given inode.
    fn mount_inner(
        &mut self,
        inode: INodeReference,
        device: Arc<dyn MountableFileSystem + Send + Sync + 'static>,
    ) {
        device.set_mount_device_id(self.devices.len() + 1);
        self.devices.push(device);
        self.mounted_filesystems
            .insert(inode, self.devices.len() - 1);
    }
}

impl Default for VirtualFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl FileSystem for VirtualFileSystem {
    async fn root_inode(&self) -> Result<INodeReference, FileSystemError> {
        if let Some(first_device) = self.devices.first() {
            first_device.root_inode().await
        } else {
            Err(FileSystemError::NoMountedFilesystem)
        }
    }

    async fn inode_data(&self, inode: INodeReference) -> Result<INodeData, FileSystemError> {
        if let Some(mounted_fs) = self.mounted_filesystems.get(&inode) {
            let mounted_root = self
                .devices
                .get(*mounted_fs)
                .ok_or(FileSystemError::BadInodeWrongDevice(inode))?
                .root_inode()
                .await?;
            self.devices
                .get(*mounted_fs)
                .ok_or(FileSystemError::BadInodeWrongDevice(inode))?
                .inode_data(mounted_root)
                .await
        } else if inode.device >= 1 {
            self.devices
                .get(inode.device - 1)
                .ok_or(FileSystemError::BadInodeWrongDevice(inode))?
                .inode_data(inode)
                .await
        } else if inode.device == 0 {
            Err(FileSystemError::BadInodeWrongDevice(inode))
        } else {
            unreachable!()
        }
    }

    async fn directory_entries(
        &self,
        inode: INodeReference,
    ) -> Result<Vec<DirectoryEntry<'_>>, FileSystemError> {
        if let Some(mounted_fs) = self.mounted_filesystems.get(&inode) {
            let mounted_root = self
                .devices
                .get(*mounted_fs)
                .ok_or(FileSystemError::BadInodeWrongDevice(inode))?
                .root_inode()
                .await?;
            self.devices
                .get(*mounted_fs)
                .ok_or(FileSystemError::BadInodeWrongDevice(inode))?
                .directory_entries(mounted_root)
                .await
        } else if inode.device >= 1 {
            self.devices
                .get(inode.device - 1)
                .ok_or(FileSystemError::BadInodeWrongDevice(inode))?
                .directory_entries(inode)
                .await
        } else if inode.device == 0 {
            Err(FileSystemError::BadInodeWrongDevice(inode))
        } else {
            unreachable!()
        }
    }

    async fn open(
        &self,
        inode: INodeReference,
    ) -> Result<Box<dyn FileDescriptor>, FileSystemError> {
        if let Some(mounted_fs) = self.mounted_filesystems.get(&inode) {
            let mounted_root = self
                .devices
                .get(*mounted_fs)
                .ok_or(FileSystemError::BadInodeWrongDevice(inode))?
                .root_inode()
                .await?;
            self.devices
                .get(*mounted_fs)
                .ok_or(FileSystemError::BadInodeWrongDevice(inode))?
                .open(mounted_root)
                .await
        } else if inode.device >= 1 {
            self.devices
                .get(inode.device - 1)
                .ok_or(FileSystemError::BadInodeWrongDevice(inode))?
                .open(inode)
                .await
        } else if inode.device == 0 {
            Err(FileSystemError::BadInodeWrongDevice(inode))
        } else {
            unreachable!()
        }
    }
    
    async fn read_to_data(&self, inode: INodeReference) -> Result<Vec<u8>, FileSystemError> {
        if let Some(mounted_fs) = self.mounted_filesystems.get(&inode) {
            let mounted_root = self
                .devices
                .get(*mounted_fs)
                .ok_or(FileSystemError::BadInodeWrongDevice(inode))?
                .root_inode()
                .await?;
            self.devices
                .get(*mounted_fs)
                .ok_or(FileSystemError::BadInodeWrongDevice(inode))?
                .read_to_data(mounted_root)
                .await
        } else if inode.device >= 1 {
            self.devices
                .get(inode.device - 1)
                .ok_or(FileSystemError::BadInodeWrongDevice(inode))?
                .read_to_data(inode)
                .await
        } else if inode.device == 0 {
            Err(FileSystemError::BadInodeWrongDevice(inode))
        } else {
            unreachable!()
        }
    }
}

impl MountingFilesystem for VirtualFileSystem {
    fn mount_filesystem(
        &mut self,
        inode: INodeReference,
        device: alloc::sync::Arc<dyn MountableFileSystem + Send + Sync + 'static>,
    ) {
        self.mount_inner(inode, device);
    }
}