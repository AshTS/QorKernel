use super::{
    enums::{Architecture, BitWidth, Endian, ObjectFileType, OsABI, ProgramHeaderType, SectionHeaderType},
    raw::{RawElfHeader, RawProgramHeader, RawSectionHeader}, flags::{ProgramHeaderFlags, SectionHeaderFlags},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgramHeader {
    pub header_type: ProgramHeaderType,
    pub flags: ProgramHeaderFlags,
    pub offset: u64,
    pub virtual_addr: u64,
    pub physical_addr: u64,
    pub file_size: u64,
    pub memory_size: u64,
    pub align: u64,
}

impl core::convert::TryFrom<RawProgramHeader> for ProgramHeader {
    type Error = ();

    fn try_from(value: RawProgramHeader) -> Result<Self, ()> {
        Ok(Self {
            header_type: ProgramHeaderType::try_from(value.p_type).map_err(|_| ())?,
            flags: ProgramHeaderFlags::new(value.p_flags),
            offset: value.p_offset,
            virtual_addr: value.p_vaddr,
            physical_addr: value.p_paddr,
            file_size: value.p_filesz,
            memory_size: value.p_memsz,
            align: value.p_align,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionHeader {
    name: u32,
    section_type: SectionHeaderType,
    flags: SectionHeaderFlags,
    virtual_addr: u64,
    offset: u64,
    size: u64,
    link: u32,
    info: u32,
    align: u64,
    entry_size: u64,
}

impl core::convert::TryFrom<RawSectionHeader> for SectionHeader {
    type Error = ();

    fn try_from(value: RawSectionHeader) -> Result<Self, ()> {
        Ok(Self {
            name: value.sh_name,
            section_type: SectionHeaderType::try_from(value.sh_type).map_err(|_| ())?,
            flags: SectionHeaderFlags::new(value.sh_flags),
            virtual_addr: value.sh_addr,
            offset: value.sh_offset,
            size: value.sh_size,
            link: value.sh_link,
            info: value.sh_info,
            align: value.sh_addralign,
            entry_size: value.sh_entsize,
        })
    }
}