use alloc::{
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use spin::RwLock;

use super::{
    DirectoryEntry, EmptyFileSystem, FileDescriptor, FileSystem, FileSystemError, INodeData,
    INodeReference, MountableFileSystem, MountingFilesystem, ParentFileSystem, PathLookup,
};

pub struct VirtualFileSystem {
    path_cache: RwLock<BTreeMap<alloc::string::String, INodeReference>>,
    rev_path_cache: RwLock<BTreeMap<INodeReference, alloc::string::String>>,
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
            path_cache: RwLock::new(BTreeMap::new()),
            rev_path_cache: RwLock::new(BTreeMap::new()),
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

impl VirtualFileSystem {
    fn insert_pairing(&self, path: &str, inode: INodeReference) {
        self.insert_pairing_owned(path.to_string(), inode);
    }

    fn insert_pairing_owned(&self, path: String, inode: INodeReference) {
        self.path_cache.write().insert(path.clone(), inode);
        self.rev_path_cache.write().insert(inode, path);
    }

    async fn lookup_inner(&self, path: &str) -> Result<INodeReference, FileSystemError> {
        let mut path = path.split('/');
        let mut inode = self.root_inode().await?;
        assert!(path.next() == Some(""));
        let mut build_path = String::from("/");
        self.insert_pairing("/", inode);

        for dir in path {
            self.insert_pairing(build_path.as_str(), inode);
            let entries = self.directory_entries(inode).await?;
            let mut found = false;
            for entry in entries {
                if entry.name == dir {
                    inode = entry.inode;
                    if !build_path.ends_with('/') {
                        build_path += "/";
                    }
                    build_path += dir;
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(FileSystemError::PathNotFound);
            }
        }

        self.insert_pairing(build_path.as_str(), inode);
        Ok(inode)
    }

    #[async_recursion::async_recursion]
    async fn inner_walk_children(
        &self,
        inode: INodeReference,
        path: &str,
    ) -> Result<usize, FileSystemError> {
        self.insert_pairing(path, inode);
        let mut total = 1;
        let inode_data = self.inode_data(inode).await?;
        if inode_data.is_directory() {
            for dir_entry in self.directory_entries(inode).await? {
                if dir_entry.name == "." || dir_entry.name == ".." {
                    continue;
                };

                /* crate::trace!(
                    "{}{}{}",
                    path,
                    if path.ends_with('/') { "" } else { "/" },
                    dir_entry.name
                ); */

                total += self
                    .inner_walk_children(
                        dir_entry.inode,
                        &alloc::format!(
                            "{}{}{}",
                            path,
                            if path.ends_with('/') { "" } else { "/" },
                            dir_entry.name
                        ),
                    )
                    .await?;
            }
        }

        Ok(total)
    }
}

#[async_trait::async_trait]
impl PathLookup for VirtualFileSystem {
    async fn lookup(&self, path: &str) -> Result<INodeReference, FileSystemError> {
        let path = if path.ends_with('/') {
            let mut chars = path.chars();
            chars.next_back();
            chars.as_str()
        } else {
            path
        };

        if let Some(path) = self.path_cache.read().get(path) {
            return Ok(*path);
        }

        let inode = self.lookup_inner(path).await?;
        Ok(inode)
    }

    async fn reverse_lookup(
        &self,
        inode: INodeReference,
    ) -> Result<Option<alloc::string::String>, FileSystemError> {
        Ok(self.rev_path_cache.read().get(&inode).cloned())
    }

    async fn invalidate_cache(&self, inode: INodeReference) -> Result<(), FileSystemError> {
        let inode_data = self.inode_data(inode).await?;
        if inode_data.is_directory() {
            for dir_entry in self.directory_entries(inode).await? {
                if dir_entry.name == "." || dir_entry.name == ".." {
                    continue;
                }

                self.invalidate_cache(dir_entry.inode).await?;
            }
        }

        if let Some(reversed) = self.reverse_lookup(inode).await? {
            self.path_cache.write().remove(&reversed);
            self.rev_path_cache.write().remove(&inode);
        }
        let read = self.path_cache.read();
        let remove = read
            .iter()
            .filter(|(_, this_inode)| **this_inode == inode)
            .map(|(path, _)| path)
            .collect::<Vec<_>>();

        for rem in remove {
            self.path_cache.write().remove(rem);
        }

        Ok(())
    }

    async fn walk_children(&self, inode: INodeReference) -> Result<usize, FileSystemError> {
        self.inner_walk_children(inode, "/").await
    }
}

impl ParentFileSystem for VirtualFileSystem {}
