use crate::utils::parser::Parser;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawElfHeader {
    pub magic: [u8; 4],
    pub class: u8,
    pub data: u8,
    pub version: u8,
    pub os_abi: u8,
    pub abi_version: u8,
    pub padding: [u8; 7],
    pub elf_type: u16,
    pub machine: u16,
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

impl RawElfHeader {
    pub fn parse(parser: &mut Parser<'_>) -> Option<Self> {
        Some(Self {
            magic: parser.take_u8_array()?,
            class: parser.take_u8()?,
            data: parser.take_u8()?,
            version: parser.take_u8()?,
            os_abi: parser.take_u8()?,
            abi_version: parser.take_u8()?,
            padding: parser.take_u8_array()?,
            elf_type: parser.take_u16()?,
            machine: parser.take_u16()?,
            version2: parser.take_u32()?,
            entry: parser.take_u64()?,
            ph_offset: parser.take_u64()?,
            sh_offset: parser.take_u64()?,
            flags: parser.take_u32()?,
            header_size: parser.take_u16()?,
            ph_entry_size: parser.take_u16()?,
            ph_entry_count: parser.take_u16()?,
            sh_entry_size: parser.take_u16()?,
            sh_entry_count: parser.take_u16()?,
            sh_str_index: parser.take_u16()?,
        })
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

impl RawProgramHeader {
    /// Parse a program header as a 64-bit ELF File
    pub fn parse64(parser: &mut Parser<'_>) -> Option<Self> {
        Some(Self {
            p_type: parser.take_u32()?,
            p_flags: parser.take_u32()?,
            p_offset: parser.take_u64()?,
            p_vaddr: parser.take_u64()?,
            p_paddr: parser.take_u64()?,
            p_filesz: parser.take_u64()?,
            p_memsz: parser.take_u64()?,
            p_align: parser.take_u64()?,
        })
    }

    /// Parse a program header as a 32-bit ELF File
    pub fn parse32(parser: &mut Parser<'_>) -> Option<Self> {
        Some(Self {
            p_type: parser.take_u32()?,
            p_offset: parser.take_u32()?.into(),
            p_vaddr: parser.take_u32()?.into(),
            p_paddr: parser.take_u32()?.into(),
            p_filesz: parser.take_u32()?.into(),
            p_memsz: parser.take_u32()?.into(),
            p_flags: parser.take_u32()?,
            p_align: parser.take_u32()?.into(),
        })
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawSectionHeader {
    pub sh_name: u32,
    pub sh_type: u32,
    pub sh_flags: u64,
    pub sh_addr: u64,
    pub sh_offset: u64,
    pub sh_size: u64,
    pub sh_link: u32,
    pub sh_info: u32,
    pub sh_addralign: u64,
    pub sh_entsize: u64,
}

impl RawSectionHeader {
    pub fn parse32(parser: &mut Parser<'_>) -> Option<Self> {
        Some(Self {
            sh_name: parser.take_u32()?,
            sh_type: parser.take_u32()?,
            sh_flags: parser.take_u32()?.into(),
            sh_addr: parser.take_u32()?.into(),
            sh_offset: parser.take_u32()?.into(),
            sh_size: parser.take_u32()?.into(),
            sh_link: parser.take_u32()?,
            sh_info: parser.take_u32()?,
            sh_addralign: parser.take_u32()?.into(),
            sh_entsize: parser.take_u32()?.into(),
        })
    }

    pub fn parse64(parser: &mut Parser<'_>) -> Option<Self> {
        Some(Self {
            sh_name: parser.take_u32()?,
            sh_type: parser.take_u32()?,
            sh_flags: parser.take_u64()?,
            sh_addr: parser.take_u64()?,
            sh_offset: parser.take_u64()?,
            sh_size: parser.take_u64()?,
            sh_link: parser.take_u32()?,
            sh_info: parser.take_u32()?,
            sh_addralign: parser.take_u64()?,
            sh_entsize: parser.take_u64()?,
        })
    }
}
