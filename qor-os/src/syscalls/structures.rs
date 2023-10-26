#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// System Call Numbers
pub enum SyscallNumber {
    Read = 0,
    Write = 1,
    Open = 2,
    Close = 3,
    Stat = 4,
    Fstat = 5,
    Lstat = 6,
    Exit = 60,
}

/// Address in userspace memory
pub struct UserspaceAddress(pub usize);

impl SyscallNumber {
    /// Take the address width syscall number, and convert it to a [`SyscallNumber`].
    pub fn from_number(number: usize) -> Option<Self> {
        match number {
            0 => Some(Self::Read),
            1 => Some(Self::Write),
            2 => Some(Self::Open),
            3 => Some(Self::Close),
            4 => Some(Self::Stat),
            5 => Some(Self::Fstat),
            6 => Some(Self::Lstat),
            60 => Some(Self::Exit),
            _ => None,
        }
    }
}