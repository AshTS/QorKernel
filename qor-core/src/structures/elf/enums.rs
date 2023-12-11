#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsABI {
    SystemV,
    HPUX,
    NetBSD,
    Linux,
    GNUHurd,
    Solaris,
    AIX,
    IRIX,
    FreeBSD,
    Tru64,
    NovellModesto,
    OpenBSD,
    OpenVMS,
    NonStopKernel,
    AROS,
    FenixOS,
    CloudABI,
    OpenVOS,
    Unknown(u8),
}

impl core::convert::From<u8> for OsABI {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::SystemV,
            0x01 => Self::HPUX,
            0x02 => Self::NetBSD,
            0x03 => Self::Linux,
            0x04 => Self::GNUHurd,
            0x06 => Self::Solaris,
            0x07 => Self::AIX,
            0x08 => Self::IRIX,
            0x09 => Self::FreeBSD,
            0x0A => Self::Tru64,
            0x0B => Self::NovellModesto,
            0x0C => Self::OpenBSD,
            0x0D => Self::OpenVMS,
            0x0E => Self::NonStopKernel,
            0x0F => Self::AROS,
            0x10 => Self::FenixOS,
            0x11 => Self::CloudABI,
            0x12 => Self::OpenVOS,
            _ => Self::Unknown(value),
        }
    }
}

impl core::convert::From<OsABI> for u8 {
    fn from(value: OsABI) -> Self {
        match value {
            OsABI::SystemV => 0x00,
            OsABI::HPUX => 0x01,
            OsABI::NetBSD => 0x02,
            OsABI::Linux => 0x03,
            OsABI::GNUHurd => 0x04,
            OsABI::Solaris => 0x06,
            OsABI::AIX => 0x07,
            OsABI::IRIX => 0x08,
            OsABI::FreeBSD => 0x09,
            OsABI::Tru64 => 0x0A,
            OsABI::NovellModesto => 0x0B,
            OsABI::OpenBSD => 0x0C,
            OsABI::OpenVMS => 0x0D,
            OsABI::NonStopKernel => 0x0E,
            OsABI::AROS => 0x0F,
            OsABI::FenixOS => 0x10,
            OsABI::CloudABI => 0x11,
            OsABI::OpenVOS => 0x12,
            OsABI::Unknown(value) => value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectFileType {
    None,
    Relocatable,
    Executable,
    SharedObject,
    Core,
    Unknown(u16),
}

impl core::convert::From<u16> for ObjectFileType {
    fn from(value: u16) -> Self {
        match value {
            0x00 => Self::None,
            0x01 => Self::Relocatable,
            0x02 => Self::Executable,
            0x03 => Self::SharedObject,
            0x04 => Self::Core,
            _ => Self::Unknown(value),
        }
    }
}

impl core::convert::From<ObjectFileType> for u16 {
    fn from(value: ObjectFileType) -> Self {
        match value {
            ObjectFileType::None => 0x00,
            ObjectFileType::Relocatable => 0x01,
            ObjectFileType::Executable => 0x02,
            ObjectFileType::SharedObject => 0x03,
            ObjectFileType::Core => 0x04,
            ObjectFileType::Unknown(value) => value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    Unspecified,
    SPARC,
    X86,
    MIPS,
    PowerPC,
    S390,
    ARM,
    SuperH,
    IA64,
    X86_64,
    AArch64,
    RISCV,
    Unknown(u16),
}

impl core::convert::From<u16> for Architecture {
    fn from(value: u16) -> Self {
        match value {
            0x00 => Self::Unspecified,
            0x02 => Self::SPARC,
            0x03 => Self::X86,
            0x08 => Self::MIPS,
            0x14 => Self::PowerPC,
            0x16 => Self::S390,
            0x28 => Self::ARM,
            0x2A => Self::SuperH,
            0x32 => Self::IA64,
            0x3E => Self::X86_64,
            0xB7 => Self::AArch64,
            0xF3 => Self::RISCV,
            _ => Self::Unknown(value),
        }
    }
}

impl core::convert::From<Architecture> for u16 {
    fn from(value: Architecture) -> Self {
        match value {
            Architecture::Unspecified => 0x00,
            Architecture::SPARC => 0x02,
            Architecture::X86 => 0x03,
            Architecture::MIPS => 0x08,
            Architecture::PowerPC => 0x14,
            Architecture::S390 => 0x16,
            Architecture::ARM => 0x28,
            Architecture::SuperH => 0x2A,
            Architecture::IA64 => 0x32,
            Architecture::X86_64 => 0x3E,
            Architecture::AArch64 => 0xB7,
            Architecture::RISCV => 0xF3,
            Architecture::Unknown(value) => value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramHeaderType {
    Null,
    Load,
    Dynamic,
    Interpreter,
    Note,
    Shlib,
    ProgramHeader,
    ThreadLocalStorage,
    Unknown(u32),
}

impl core::convert::From<u32> for ProgramHeaderType {
    fn from(value: u32) -> Self {
        match value {
            0x00 => Self::Null,
            0x01 => Self::Load,
            0x02 => Self::Dynamic,
            0x03 => Self::Interpreter,
            0x04 => Self::Note,
            0x05 => Self::Shlib,
            0x06 => Self::ProgramHeader,
            0x07 => Self::ThreadLocalStorage,
            _ => Self::Unknown(value),
        }
    }
}

impl core::convert::From<ProgramHeaderType> for u32 {
    fn from(value: ProgramHeaderType) -> Self {
        match value {
            ProgramHeaderType::Null => 0x00,
            ProgramHeaderType::Load => 0x01,
            ProgramHeaderType::Dynamic => 0x02,
            ProgramHeaderType::Interpreter => 0x03,
            ProgramHeaderType::Note => 0x04,
            ProgramHeaderType::Shlib => 0x05,
            ProgramHeaderType::ProgramHeader => 0x06,
            ProgramHeaderType::ThreadLocalStorage => 0x07,
            ProgramHeaderType::Unknown(value) => value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionHeaderType {
    Null,
    ProgBits,
    SymTab,
    StrTab,
    Rela,
    Hash,
    Dynamic,
    Note,
    NoBits,
    Rel,
    Shlib,
    DynSym,
    Unknown(u32),
}

impl core::convert::From<u32> for SectionHeaderType {
    fn from(value: u32) -> Self {
        match value {
            0x00 => Self::Null,
            0x01 => Self::ProgBits,
            0x02 => Self::SymTab,
            0x03 => Self::StrTab,
            0x04 => Self::Rela,
            0x05 => Self::Hash,
            0x06 => Self::Dynamic,
            0x07 => Self::Note,
            0x08 => Self::NoBits,
            0x09 => Self::Rel,
            0x0A => Self::Shlib,
            0x0B => Self::DynSym,
            _ => Self::Unknown(value),
        }
    }
}

impl core::convert::From<SectionHeaderType> for u32 {
    fn from(value: SectionHeaderType) -> Self {
        match value {
            SectionHeaderType::Null => 0x00,
            SectionHeaderType::ProgBits => 0x01,
            SectionHeaderType::SymTab => 0x02,
            SectionHeaderType::StrTab => 0x03,
            SectionHeaderType::Rela => 0x04,
            SectionHeaderType::Hash => 0x05,
            SectionHeaderType::Dynamic => 0x06,
            SectionHeaderType::Note => 0x07,
            SectionHeaderType::NoBits => 0x08,
            SectionHeaderType::Rel => 0x09,
            SectionHeaderType::Shlib => 0x0A,
            SectionHeaderType::DynSym => 0x0B,
            SectionHeaderType::Unknown(value) => value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    Little,
    Big,
}

impl core::convert::TryFrom<u8> for Endian {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, u8> {
        match value {
            0x01 => Ok(Self::Little),
            0x02 => Ok(Self::Big),
            _ => Err(value),
        }
    }
}

impl core::convert::From<Endian> for u8 {
    fn from(value: Endian) -> Self {
        match value {
            Endian::Little => 0x01,
            Endian::Big => 0x02,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitWidth {
    Bit32,
    Bit64,
}

impl core::convert::TryFrom<u8> for BitWidth {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, u8> {
        match value {
            0x01 => Ok(Self::Bit32),
            0x02 => Ok(Self::Bit64),
            _ => Err(value),
        }
    }
}

impl core::convert::From<BitWidth> for u8 {
    fn from(value: BitWidth) -> Self {
        match value {
            BitWidth::Bit32 => 0x01,
            BitWidth::Bit64 => 0x02,
        }
    }
}
