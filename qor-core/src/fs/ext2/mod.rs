use crate::{drivers::block::BlockDeviceDriver, sync::Mutex, interfaces::fs::{FileSystem, INodeReference, FileSystemError, INodeData, FileDescriptor, MountableFileSystem}, utils::rawstr::OsStrRef, structures::{time::UnixTimestamp, id::{UserID, GroupID}}};

use self::raw::{SuperBlock, Inode, DirectoryEntry};

pub mod raw;

const fn div_ceil(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

#[allow(clippy::module_name_repetitions)]
pub struct Ext2FileSystem<E: 'static + core::fmt::Debug + Send + Sync> {
    device_id: core::sync::atomic::AtomicUsize,
    device: &'static (dyn BlockDeviceDriver<512, E, u32> + Send + Sync),
    cached_super_block: Mutex<Option<SuperBlock>>,
}

impl<E: 'static + core::fmt::Debug + Send + Sync> Ext2FileSystem<E> {
    pub fn new(device: &'static (dyn BlockDeviceDriver<512, E, u32> + Send + Sync)) -> Self {
        Self {
            device_id: 0.into(),
            device,
            cached_super_block: Mutex::new(None),
        }
    }

    /// Returns the read super block of this [`Ext2FileSystem<E>`].
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the super block could not be read.
    ///
    /// # Panics
    ///
    /// This function will panic if the internal buffer is improperly sized.
    pub async fn read_super_block(&self) -> Result<SuperBlock, E> {
        let lock = self.cached_super_block.async_lock().await;
        if let Some(cached_super_block) = lock.as_ref() {
            Ok(*cached_super_block)
        }
        else {
            core::mem::drop(lock);
            
            let mut buffer = [0; 1024];
            self.read_kb_block(1, &mut buffer).await?;
        
            let super_block = SuperBlock::from_bytes(&buffer);
            self.cached_super_block.async_lock().await.replace(super_block);
            Ok(super_block)
        }
    }

    /// Read a given block from the block device.
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the block could not be read.
    pub async fn read_kb_block<'a>(&self, block: u32, buffer: &'a mut [u8; 1024]) -> Result<&'a mut [u8; 1024], E> {
        let mut inner_buffer = [[0u8; 512]; 2];

        self.device.read_blocks(2 * block, &mut inner_buffer).await?;

        *buffer = unsafe { core::mem::transmute(inner_buffer) };
        Ok(buffer)
    }

    /// Read a given block from the block device.
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the block could not be read.
    /// 
    /// # Panics
    /// 
    /// This function will panic if the block index cannot fit within a `u32` or if the buffer is not the proper length.
    pub async fn read_block<'a>(&self, block: u32, buffer: &'a mut [u8]) -> Result<&'a mut [u8], E> {
        let sb = self.read_super_block().await?;
        let block_size = sb.block_size(); // We know this will be a multiple of a KiB because the block size is stored
                                                 // as log base 2 of the block size in bytes minus 1024.
        let block_size_kib = block_size / 1024;

        let block_index = block as usize * block_size_kib;

        for kib_index in 0..block_size_kib {
            let mut buf = [0; 1024];
            self.read_kb_block((block_index + kib_index).try_into().unwrap(), &mut buf).await?;
            buffer[1024 * kib_index..1024 * (kib_index + 1)].copy_from_slice(&buf);
        }

        Ok(buffer)
    }

    /// Read a given block from the block device.
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the block could not be read.
    /// 
    /// # Panics
    /// 
    /// This function will panic if the block index cannot fit within a `u32` or if the buffer is not the proper length.
    pub async fn read_block_alloc<'a>(&self, block: u32) -> Result<alloc::vec::Vec<u8>, E> {
        let sb = self.read_super_block().await?;
        let block_size = sb.block_size(); // We know this will be a multiple of a KiB because the block size is stored
                                                 // as log base 2 of the block size in bytes minus 1024.
        let block_size_kib = block_size / 1024;

        let block_index = block as usize * block_size_kib;

        let mut buffer = alloc::vec![0; block_size];

        for kib_index in 0..block_size_kib {
            let mut buf = [0; 1024];
            self.read_kb_block((block_index + kib_index).try_into().unwrap(), &mut buf).await?;
            buffer[1024 * kib_index..1024 * (kib_index + 1)].copy_from_slice(&buf);
        }

        Ok(buffer)
    }

    /// Read enough blocks to fill the given buffer up to a KiB boundary.
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the block could not be read.
    /// 
    /// # Panics
    /// 
    /// This function will panic if the block index cannot fit within a `u32` or if the buffer is not the proper length.
    pub async fn read_blocks<'a>(&self, block: u32, buffer: &'a mut [u8]) -> Result<&'a mut [u8], E> {
        let sb = self.read_super_block().await?;
        let block_size = sb.block_size(); // We know this will be a multiple of a KiB because the block size is stored
                                                 // as log base 2 of the block size in bytes minus 1024.
        let block_size_kib = block_size / 1024;

        let block_index = block as usize * block_size_kib;

        let size_kib = buffer.len() / 1024;

        for kib_index in 0..size_kib {
            let mut buf = [0; 1024];
            self.read_kb_block((block_index + kib_index).try_into().unwrap(),
            &mut buf).await?;

            buffer[1024 * kib_index..1024 * (kib_index + 1)].copy_from_slice(&buf);
        }

        Ok(buffer)
    }

    /// Read a block group descriptor with the given index.
    ///
    /// # Errors
    /// 
    /// This function will return an error if the block group descriptor could not be read.
    /// 
    /// # Panics
    /// 
    /// This function will panic if the block index cannot fit within a `u32` or if the buffer is not the proper length.
    pub async fn block_group_descriptor(&self, index: usize) -> Result<raw::BlockGroupDescriptor, E> {
        let sb = self.read_super_block().await?;
        let desc_count = sb.block_group_count();

        let desc_size = 64;
        let buffer_length = div_ceil(desc_count * desc_size, 1024) * 1024;
        let mut buffer = alloc::vec![0; buffer_length];

        self.read_blocks(sb.block_group_descriptor_table_index().try_into().unwrap(), buffer.as_mut_slice()).await?;

        // Chunk the buffer into descriptor table sized chunks
        let mut chunks = buffer.chunks_exact(desc_size);
        Ok(raw::BlockGroupDescriptor::from_bytes(chunks.nth(index).unwrap().try_into().unwrap()))
    }

    /// Get an inode from the block device.
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the inode could not be read.
    /// 
    /// # Panics
    /// 
    /// This function will panic if the block index of the inode cannot fit within a `u32`.
    pub async fn get_inode(&self, inode_index: u32) -> Result<Inode, E> {
        // Inodes start at zero
        assert!(inode_index > 0);
        let inode_index = inode_index - 1;

        let sb = self.read_super_block().await?;

        let inode_size = sb.extended.map_or(128, |ext| ext.inode_structure_size) as usize;

        let block_group_index = inode_index as usize / sb.inodes_per_block_group as usize;
        let inode_index_in_group = inode_index as usize % sb.inodes_per_block_group as usize;

        let inodes_per_block = sb.block_size() / inode_size;

        let descriptor = self.block_group_descriptor(block_group_index).await?;
        let block_index_start = descriptor.starting_block_inode_table as usize;
        let block_index = block_index_start + inode_index_in_group / inodes_per_block;
        let index_in_block = inode_index_in_group % inodes_per_block;

        let mut buffer = alloc::vec![0; sb.block_size()];
        self.read_block(block_index.try_into().unwrap(), buffer.as_mut_slice()).await?;

        let mut chunks = buffer.chunks_exact(inode_size);
        Ok(Inode::from_bytes(chunks.nth(index_in_block).unwrap()))
    }

    /// Read data from an inode.
    ///
    /// # Errors
    /// 
    /// This function will return an error if the data could not be read from the inode.
    pub async fn read_inode_data(&self, inode: &Inode, buffer: &mut [u8]) -> Result<(), E> {
        let block_index = inode.block_pointers[0];
        self.read_block(block_index, buffer).await?;

        Ok(())
    }

    /// Read directory entries from an inode.
    ///
    /// # Errors
    /// 
    /// This function will return an error if the data could not be read from the inode or disk.
    pub async fn read_directory_entries(&self, inode: &Inode) -> Result<alloc::vec::Vec<DirectoryEntry>, E> {
        let mut buffer = alloc::vec![0; inode.size(true)];
        
        self.read_inode_data(inode, &mut buffer).await?;

        Ok(DirectoryEntry::from_bytes(buffer.as_slice()))
    }
}

use alloc::{boxed::Box, string::ToString};

#[async_trait::async_trait]
impl<E: core::fmt::Debug + Send + Sync> FileSystem for Ext2FileSystem<E> {
    async fn root_inode(&self) -> Result<INodeReference, FileSystemError> {
        Ok(INodeReference { inode: 2, device: self.device_id.load(core::sync::atomic::Ordering::Acquire) })
    }
    
    async fn inode_data(&self, inode: INodeReference) -> Result<INodeData, FileSystemError> {
        let inner = self.get_inode(inode.inode.try_into().unwrap()).await.map_err(|_| FileSystemError::BadInode(inode))?;

        Ok(INodeData {
            mode: inner.mode.into(),
            link_count: inner.hard_link_count as usize,
            uid: UserID(inner.user_id),
            gid: GroupID(inner.group_id),
            size: inner.size(true),
            access_time: UnixTimestamp(u64::from(inner.last_access_time)),
            modify_time: UnixTimestamp(u64::from(inner.last_modify_time)),
            change_time: UnixTimestamp(u64::from(inner.last_modify_time)),
            reference: inode,
        })
    }

    async fn directory_entries(
        &self,
        inode: INodeReference,
    ) -> Result<alloc::vec::Vec<crate::interfaces::fs::DirectoryEntry<'_>>, FileSystemError> {
        // Get this device id
        let device_id = self.device_id.load(core::sync::atomic::Ordering::Acquire);

        // First, read the inode structure from disk
        let inode_data = self.get_inode(inode.inode.try_into().unwrap()).await.map_err(|_| FileSystemError::BadInode(inode))?;

        // Next, load the directory entries
        let directory_entries = self.read_directory_entries(&inode_data).await.map_err(|_| FileSystemError::BadInode(inode))?;

        // Finally, we map to the regular struct
        Ok(directory_entries.iter().map(|entry| 
            crate::interfaces::fs::DirectoryEntry{
                inode: INodeReference { inode: entry.inode as usize, device: device_id} ,
            name: OsStrRef::new(&entry.name).to_string().into() }).collect())
    }

    async fn open(&self, _inode: INodeReference)
        -> Result<Box<dyn FileDescriptor>, FileSystemError> {
        todo!()
    }
}

impl<E: core::fmt::Debug + Send + Sync> MountableFileSystem for Ext2FileSystem<E> {
    fn set_mount_device_id(&self, device_id: usize) {
        self.device_id.store(device_id, core::sync::atomic::Ordering::Release);
    }
}