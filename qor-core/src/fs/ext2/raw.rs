const fn div_ceil(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

pub struct Parser<'a> {
    data: &'a [u8],
}

impl<'a> Parser<'a> {
    /// Construct a new parser for a slice of `u8`'s
    #[must_use]
    pub const fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Take a `u8` from the slice.
    #[must_use]
    pub fn take_u8(&mut self) -> Option<u8> {
        if self.data.is_empty() {
            None
        } else {
            let result = self.data[0];
            self.data = &self.data[1..];

            Some(result)
        }
    }

    /// Take a `u16` from the slice.
    ///
    /// # Panics
    ///
    /// This function will panic if the slice is of the wrong size.
    #[must_use]
    pub fn take_u16(&mut self) -> Option<u16> {
        if self.data.len() >= 2 {
            let result = u16::from_le_bytes(self.data[0..2].try_into().unwrap());
            self.data = &self.data[2..];

            Some(result)
        } else {
            None
        }
    }

    /// Take a `u32` from the slice.
    ///
    /// # Panics
    ///
    /// This function will panic if the slice is of the wrong size.
    #[must_use]
    pub fn take_u32(&mut self) -> Option<u32> {
        if self.data.len() >= 4 {
            let result = u32::from_le_bytes(self.data[0..4].try_into().unwrap());
            self.data = &self.data[4..];

            Some(result)
        } else {
            None
        }
    }

    /// Take a `u64` from the slice.
    ///
    /// # Panics
    ///
    /// This function will panic if the slice is of the wrong size.
    #[must_use]
    pub fn take_u64(&mut self) -> Option<u64> {
        if self.data.len() >= 8 {
            let result = u64::from_le_bytes(self.data[0..8].try_into().unwrap());
            self.data = &self.data[8..];

            Some(result)
        } else {
            None
        }
    }

    /// Take a `u128` from the slice.
    ///
    /// # Panics
    ///
    /// This function will panic if the slice is of the wrong size.
    #[must_use]
    pub fn take_u128(&mut self) -> Option<u128> {
        if self.data.len() >= 16 {
            let result = u128::from_le_bytes(self.data[0..16].try_into().unwrap());
            self.data = &self.data[16..];

            Some(result)
        } else {
            None
        }
    }

    /// Take an array of `u8`'s of a given length.
    ///
    /// # Panics
    ///
    /// This function will panic if the slice is of the wrong size.
    pub fn take_u8_array<const L: usize>(&mut self) -> Option<[u8; L]> {
        if self.data.len() >= L {
            let result = self.data[0..L].try_into().unwrap();
            self.data = &self.data[L..];

            Some(result)
        } else {
            None
        }
    }

    /// Take an array of `u32`'s of a given length.
    ///
    /// # Panics
    ///
    /// This function will panic if the slice is of the wrong size.
    pub fn take_u32_array<const L: usize>(&mut self) -> Option<[u32; L]> {
        if self.data.len() >= L {
            let mut result = [0; L];
            for slot in result.iter_mut() {
                *slot = self.take_u32().unwrap();
            }

            Some(result)
        } else {
            None
        }
    }

    /// Skip a certain number of bytes
    pub fn skip(&mut self, length: usize) -> Option<()> {
        if self.data.len() >= length {
            self.data = &self.data[length..];

            Some(())
        } else {
            None
        }
    }

    /// Returns true if the buffer is empty
    #[must_use]
    pub const fn empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl<'a> core::iter::Iterator for Parser<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.take_u8()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtendedSuperblock {
    pub first_unreserved_inode: u32,
    pub inode_structure_size: u16,
    pub block_group_for_super_block: u16,
    pub optional_features: u32,
    pub required_features: u32,
    pub read_only_features: u32,
    pub file_system_id: [u8; 16],
    pub volume_name: [u8; 16],
    pub path_volume_of_last_mount: [u8; 64],
    pub compression_algorithm: u32,
    pub blocks_for_file_prealloc: u32,
    pub blocks_for_directory_prealloc: u32,
    pub journal_id: u128,
    pub journal_inode: u32,
    pub journal_device: u32,
    pub orphan_inode_list_head: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuperBlock {
    pub inode_count: u32,
    pub block_count: u32,
    pub super_user_blocks: u32,
    pub unallocated_blocks: u32,
    pub unallocated_inodes: u32,
    pub super_block_block_number: u32,
    pub block_size_log_2_less_10: u32,
    pub fragment_size_log_2_less_10: u32,
    pub blocks_per_block_group: u32,
    pub fragments_per_block_group: u32,
    pub inodes_per_block_group: u32,
    pub last_mount_time: u32,
    pub last_write_time: u32,
    pub mounts_since_consistency_check: u16,
    pub mounts_until_consistency_check: u16,
    pub ext2_signature: u16,
    pub file_system_state: u16,
    pub error_handle_mode: u16,
    pub minor_version: u16,
    pub last_consistency_check: u32,
    pub interval_between_consistency_check: u32,
    pub operating_system_id: u32,
    pub major_version: u32,
    pub user_id_for_reserved: u16,
    pub group_id_for_reserved: u16,

    pub extended: Option<ExtendedSuperblock>,
}

impl SuperBlock {
    /// Parses a Ext2 Super Block from a byte buffer.
    ///
    /// # Panics
    ///
    /// Panics if the buffer passed is an invalid size.
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut parser = Parser::new(bytes);

        let inode_count = parser.take_u32().unwrap();
        let block_count = parser.take_u32().unwrap();
        let super_user_blocks = parser.take_u32().unwrap();
        let unallocated_blocks = parser.take_u32().unwrap();
        let unallocated_inodes = parser.take_u32().unwrap();
        let super_block_block_number = parser.take_u32().unwrap();
        let block_size_log_2_less_10 = parser.take_u32().unwrap();
        let fragment_size_log_2_less_10 = parser.take_u32().unwrap();
        let blocks_per_block_group = parser.take_u32().unwrap();
        let fragments_per_block_group = parser.take_u32().unwrap();
        let inodes_per_block_group = parser.take_u32().unwrap();
        let last_mount_time = parser.take_u32().unwrap();
        let last_write_time = parser.take_u32().unwrap();
        let mounts_since_consistency_check = parser.take_u16().unwrap();
        let mounts_until_consistency_check = parser.take_u16().unwrap();
        let ext2_signature = parser.take_u16().unwrap();
        let file_system_state = parser.take_u16().unwrap();
        let error_handle_mode = parser.take_u16().unwrap();
        let minor_version = parser.take_u16().unwrap();
        let last_consistency_check = parser.take_u32().unwrap();
        let interval_between_consistency_check = parser.take_u32().unwrap();
        let operating_system_id = parser.take_u32().unwrap();
        let major_version = parser.take_u32().unwrap();
        let user_id_for_reserved = parser.take_u16().unwrap();
        let group_id_for_reserved = parser.take_u16().unwrap();

        let extended = if major_version >= 1 {
            let first_unreserved_inode = parser.take_u32().unwrap();
            let inode_structure_size = parser.take_u16().unwrap();
            let block_group_for_super_block = parser.take_u16().unwrap();
            let optional_features = parser.take_u32().unwrap();
            let required_features = parser.take_u32().unwrap();
            let read_only_features = parser.take_u32().unwrap();
            let file_system_id = parser.take_u8_array().unwrap();
            let volume_name = parser.take_u8_array().unwrap();
            let path_volume_of_last_mount = parser.take_u8_array().unwrap();
            let compression_algorithm = parser.take_u32().unwrap();
            let blocks_for_file_prealloc = parser.take_u32().unwrap();
            let blocks_for_directory_prealloc = parser.take_u32().unwrap();
            let journal_id = parser.take_u128().unwrap();
            let journal_inode = parser.take_u32().unwrap();
            let journal_device = parser.take_u32().unwrap();
            let orphan_inode_list_head = parser.take_u32().unwrap();

            Some(ExtendedSuperblock {
                first_unreserved_inode,
                inode_structure_size,
                block_group_for_super_block,
                optional_features,
                required_features,
                read_only_features,
                file_system_id,
                volume_name,
                path_volume_of_last_mount,
                compression_algorithm,
                blocks_for_file_prealloc,
                blocks_for_directory_prealloc,
                journal_id,
                journal_inode,
                journal_device,
                orphan_inode_list_head,
            })
        } else {
            None
        };

        Self {
            inode_count,
            block_count,
            super_user_blocks,
            unallocated_blocks,
            unallocated_inodes,
            super_block_block_number,
            block_size_log_2_less_10,
            fragment_size_log_2_less_10,
            blocks_per_block_group,
            fragments_per_block_group,
            inodes_per_block_group,
            last_mount_time,
            last_write_time,
            mounts_since_consistency_check,
            mounts_until_consistency_check,
            ext2_signature,
            file_system_state,
            error_handle_mode,
            minor_version,
            last_consistency_check,
            interval_between_consistency_check,
            operating_system_id,
            major_version,
            user_id_for_reserved,
            group_id_for_reserved,
            extended,
        }
    }

    #[must_use]
    pub const fn block_size(&self) -> usize {
        1024 << self.block_size_log_2_less_10
    }

    #[must_use]
    pub const fn block_group_count(&self) -> usize {
        let block_group_count_by_block = div_ceil(
            self.block_count as usize,
            self.blocks_per_block_group as usize,
        );
        let block_group_count_by_inode = div_ceil(
            self.inode_count as usize,
            self.inodes_per_block_group as usize,
        );

        if block_group_count_by_block >= block_group_count_by_inode {
            block_group_count_by_block
        } else {
            block_group_count_by_inode
        }
    }

    #[must_use]
    pub const fn block_group_descriptor_table_index(&self) -> usize {
        if self.block_size_log_2_less_10 == 0 {
            2
        } else {
            1
        }
    }

    #[must_use]
    pub const fn use_64_bit_sizes(&self) -> bool {
        if let Some(extended) = self.extended {
            extended.read_only_features & 2 > 0
        } else {
            false
        }
    }
}

/// Minix3 Inode
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct Inode {
    pub mode: u16,
    pub user_id: u16,
    pub lower_32_size: u32,
    pub last_access_time: u32,
    pub create_time: u32,
    pub last_modify_time: u32,
    pub delete_time: u32,
    pub group_id: u16,
    pub hard_link_count: u16,
    pub disk_sectors: u32,
    pub flags: u32,
    pub os_specific_1: u32,
    pub block_pointers: [u32; 15],
    pub generation_number: u32,
    pub extended_attribute_block: u32,
    pub upper_32_size: u32,
    pub fragment_block_address: u32,
    pub os_specific_2: [u8; 12],
}

impl Inode {
    /// Parses a Minix3 Inode from a byte buffer.
    ///
    /// # Panics
    ///
    /// Panics if the buffer passed is an invalid size.
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut parser = Parser::new(bytes);

        let mode = parser.take_u16().unwrap();
        let user_id = parser.take_u16().unwrap();
        let lower_32_size = parser.take_u32().unwrap();
        let last_access_time = parser.take_u32().unwrap();
        let create_time = parser.take_u32().unwrap();
        let last_modify_time = parser.take_u32().unwrap();
        let delete_time = parser.take_u32().unwrap();
        let group_id = parser.take_u16().unwrap();
        let hard_link_count = parser.take_u16().unwrap();
        let disk_sectors = parser.take_u32().unwrap();
        let flags = parser.take_u32().unwrap();
        let os_specific_1 = parser.take_u32().unwrap();
        let block_pointers = parser.take_u32_array().unwrap();
        let generation_number = parser.take_u32().unwrap();
        let extended_attribute_block = parser.take_u32().unwrap();
        let upper_32_size = parser.take_u32().unwrap();
        let fragment_block_address = parser.take_u32().unwrap();
        let os_specific_2 = parser.take_u8_array().unwrap();

        Self {
            mode,
            user_id,
            lower_32_size,
            last_access_time,
            create_time,
            last_modify_time,
            delete_time,
            group_id,
            hard_link_count,
            disk_sectors,
            flags,
            os_specific_1,
            block_pointers,
            generation_number,
            extended_attribute_block,
            upper_32_size,
            fragment_block_address,
            os_specific_2,
        }
    }

    #[must_use]
    pub const fn size(&self, use_extended: bool) -> usize {
        if use_extended {
            (self.lower_32_size as usize) | ((self.upper_32_size as usize) << 32)
        } else {
            self.lower_32_size as usize
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockGroupDescriptor {
    pub block_usage_bitmap: u32,
    pub inode_usage_bitmap: u32,
    pub starting_block_inode_table: u32,
    pub remaining_unallocated_blocks: u16,
    pub remaining_unallocated_inodes: u16,
    pub directories_in_group: u16,
}

impl BlockGroupDescriptor {
    /// Parses a Block Group Descriptor
    ///
    /// # Panics
    ///
    /// Panics if the buffer passed is an invalid size.
    #[must_use]
    pub fn from_bytes(bytes: &[u8; 64]) -> Self {
        let mut parser = Parser::new(bytes);

        let block_usage_bitmap = parser.take_u32().unwrap();
        let inode_usage_bitmap = parser.take_u32().unwrap();
        let starting_block_inode_table = parser.take_u32().unwrap();
        let remaining_unallocated_blocks = parser.take_u16().unwrap();
        let remaining_unallocated_inodes = parser.take_u16().unwrap();
        let directories_in_group = parser.take_u16().unwrap();

        Self {
            block_usage_bitmap,
            inode_usage_bitmap,
            starting_block_inode_table,
            remaining_unallocated_blocks,
            remaining_unallocated_inodes,
            directories_in_group,
        }
    }
}

pub struct DirectoryEntry {
    pub inode: u32,
    pub name: alloc::vec::Vec<u8>,
}

impl DirectoryEntry {
    /// Parses directory entries from a buffer
    ///
    /// # Panics
    ///
    /// Panics if the buffer passed is an invalid size.
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> alloc::vec::Vec<Self> {
        let mut parser = Parser::new(bytes);
        let mut result = alloc::vec::Vec::new();

        while !parser.empty() {
            let inode = parser.take_u32().unwrap();
            let total_size = parser.take_u16().unwrap();
            let _ = parser.take_u8(); // Skip name length
            let _ = parser.take_u8(); // Skip type indicator
            let name = (&mut parser)
                .take(total_size as usize - 8)
                .collect::<alloc::vec::Vec<_>>();

            result.push(Self { inode, name });
        }

        result
    }
}
