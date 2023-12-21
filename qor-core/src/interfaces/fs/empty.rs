use super::{
    DirectoryEntry, FileDescriptor, FileSystem, FileSystemError, INodeData, INodeReference,
    MountableFileSystem, SeekMode,
};

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct EmptyFileSystem {
    id: core::sync::atomic::AtomicUsize,
}

#[allow(clippy::module_name_repetitions)]
pub struct EmptyFileDescriptor {}

impl EmptyFileSystem {
    /// Creates a new [`EmptyFileSystem`].
    #[must_use]
    pub const fn new() -> Self {
        Self {
            id: core::sync::atomic::AtomicUsize::new(42),
        }
    }

    /// Construct an [`INodeReference`] from a given inode number.
    fn inode_ref(&self, inode: usize) -> super::INodeReference {
        super::INodeReference {
            inode,
            device: self.id.load(core::sync::atomic::Ordering::Acquire),
        }
    }

    /// Verify that the [`INodeReference`] belongs to this device.
    fn verify_ref(&self, inode_ref: INodeReference) -> Result<(), FileSystemError> {
        if inode_ref.device == self.id.load(core::sync::atomic::Ordering::Acquire) {
            Ok(())
        } else {
            Err(FileSystemError::BadInodeWrongDevice(inode_ref))
        }
    }
}

impl Default for EmptyFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl FileSystem for EmptyFileSystem {
    async fn root_inode(&self) -> Result<INodeReference, FileSystemError> {
        Ok(self.inode_ref(0))
    }

    async fn inode_data(&self, inode: INodeReference) -> Result<INodeData, FileSystemError> {
        self.verify_ref(inode)?;
        match inode.inode {
            0 => Ok(INodeData {
                mode: 0.into(),
                link_count: 0,
                uid: 0.into(),
                gid: 0.into(),
                size: 0,
                access_time: 0.into(),
                modify_time: 0.into(),
                change_time: 0.into(),
                reference: inode,
            }),
            _ => Err(FileSystemError::BadInode(inode)),
        }
    }

    async fn directory_entries(
        &self,
        inode: INodeReference,
    ) -> Result<Vec<DirectoryEntry<'_>>, FileSystemError> {
        self.verify_ref(inode)?;
        match inode.inode {
            0 => Ok(alloc::vec![
                DirectoryEntry {
                    inode: self.inode_ref(0),
                    name: ".".into()
                },
                DirectoryEntry {
                    inode: self.inode_ref(0),
                    name: "..".into()
                }
            ]),
            _ => Err(FileSystemError::BadInode(inode)),
        }
    }

    async fn open(
        &self,
        inode: INodeReference,
    ) -> Result<Arc<dyn FileDescriptor>, FileSystemError> {
        self.verify_ref(inode)?;
        match inode.inode {
            0 => Ok(Arc::new(EmptyFileDescriptor {})),
            _ => Err(FileSystemError::BadInode(inode)),
        }
    }

    async fn read_to_data(&self, inode: INodeReference) -> Result<Vec<u8>, FileSystemError> {
        self.verify_ref(inode)?;
        match inode.inode {
            0 => Ok(alloc::vec::Vec::new()),
            _ => Err(FileSystemError::BadInode(inode)),
        }
    }
}

impl MountableFileSystem for EmptyFileSystem {
    fn set_mount_device_id(&self, device_id: usize) {
        self.id
            .store(device_id, core::sync::atomic::Ordering::Release);
    }
}

#[async_trait::async_trait]
impl FileDescriptor for EmptyFileDescriptor {
    /// Read bytes from the file starting at the cursor into `buffer`. Returns the number of bytes read.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation failed.
    async fn read(&self, buffer: &mut [u8]) -> Result<usize, FileSystemError> {
        buffer.fill(0);
        Ok(buffer.len())
    }

    /// Writes bytes to the file starting at the cursor. Returns the number of bytes written.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation failed.
    async fn write(&self, buffer: &[u8]) -> Result<usize, FileSystemError> {
        Ok(buffer.len())
    }

    /// Seeks to a position in the file.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation failed.
    async fn seek(&self, _seek: SeekMode) -> Result<usize, FileSystemError> {
        Ok(0)
    }
}
