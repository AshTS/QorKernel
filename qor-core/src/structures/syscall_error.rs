#[derive(Debug)]
pub enum SyscallError {
    BadFileDescriptor,
    Fault
}

impl core::convert::From<SyscallError> for isize {
    fn from(value: SyscallError) -> Self {
        match value {
            SyscallError::BadFileDescriptor => 9,
            SyscallError::Fault => 14,
        }
    }
}