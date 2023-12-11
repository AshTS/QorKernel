use alloc::vec::Vec;

use crate::utils::parser::Parser;

pub mod enums;
pub mod flags;
pub mod raw;
pub mod structures;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Elf<'a> {
    pub header: structures::ElfHeader,
    pub program_headers: Vec<raw::RawProgramHeader>,
    pub section_headers: Vec<raw::RawSectionHeader>,
    pub data: &'a [u8],
}

impl<'a> Elf<'a> {
    #[must_use]
    /// Parse an ELF file from a slice of bytes
    ///
    /// # Panics
    ///
    /// This function will panic if the ELF file is malformed or a 64 bit file with > 32 bit pointers is attempted to
    /// be read on a 32 bit machine.
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut parser = Parser::new(data);

        let header = raw::RawElfHeader::parse(&mut parser)?;

        let program_header_offset = header.ph_offset.try_into().unwrap();
        let program_header_size = header.ph_entry_size as usize * header.ph_entry_count as usize;

        let section_header_offset = header.sh_offset.try_into().unwrap();
        let section_header_size = header.sh_entry_size as usize * header.sh_entry_count as usize;

        let mut parser = Parser::new(
            data[program_header_offset..program_header_offset + program_header_size].as_ref(),
        );
        let program_headers = (0..header.ph_entry_count)
            .map(|_| {
                if header.data > 1 {
                    raw::RawProgramHeader::parse64(&mut parser)
                } else {
                    raw::RawProgramHeader::parse32(&mut parser)
                }
            })
            .collect::<Option<Vec<_>>>()?;

        let mut parser = Parser::new(
            data[section_header_offset..section_header_offset + section_header_size].as_ref(),
        );
        let section_headers = (0..header.sh_entry_count)
            .map(|_| {
                if header.data > 1 {
                    raw::RawSectionHeader::parse64(&mut parser)
                } else {
                    raw::RawSectionHeader::parse32(&mut parser)
                }
            })
            .collect::<Option<Vec<_>>>()?;

        Some(Self {
            header: header.try_into().unwrap(),
            program_headers,
            section_headers,
            data,
        })
    }
}
