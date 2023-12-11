use super::{
    enums::{Architecture, BitWidth, Endian, ObjectFileType, OsABI},
    raw::RawElfHeader,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElfHeader {
    pub class: BitWidth,
    pub endian: Endian,
    pub version: u8,
    pub os_abi: OsABI,
    pub abi_version: u8,
    pub elf_type: ObjectFileType,
    pub machine: Architecture,
    pub version2: u32,
    pub entry: u64,
    pub ph_offset: u64,
    pub sh_offset: u64,
    pub flags: u32,
    pub header_size: u16,
    pub ph_entry_size: u16,
    pub ph_entry_count: u16,
    pub sh_entry_size: u16,
    pub sh_entry_count: u16,
    pub sh_str_index: u16,
}

impl core::convert::TryFrom<RawElfHeader> for ElfHeader {
    type Error = ();

    fn try_from(value: RawElfHeader) -> Result<Self, ()> {
        if value.magic != [0x7F, 0x45, 0x4C, 0x46] {
            return Err(());
        }

        Ok(Self {
            class: BitWidth::try_from(value.class).map_err(|_| ())?,
            endian: Endian::try_from(value.data).map_err(|_| ())?,
            version: value.version,
            os_abi: OsABI::try_from(value.os_abi).map_err(|_| ())?,
            abi_version: value.abi_version,
            elf_type: ObjectFileType::try_from(value.elf_type).map_err(|_| ())?,
            machine: Architecture::try_from(value.machine).map_err(|_| ())?,
            version2: value.version2,
            entry: value.entry,
            ph_offset: value.ph_offset,
            sh_offset: value.sh_offset,
            flags: value.flags,
            header_size: value.header_size,
            ph_entry_size: value.ph_entry_size,
            ph_entry_count: value.ph_entry_count,
            sh_entry_size: value.sh_entry_size,
            sh_entry_count: value.sh_entry_count,
            sh_str_index: value.sh_str_index,
        })
    }
}
