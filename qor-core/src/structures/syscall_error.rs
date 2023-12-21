#[derive(Debug)]
pub enum SyscallError {
    Fault
}

impl core::convert::From<SyscallError> for isize {
    fn from(value: SyscallError) -> Self {
        match value {
            SyscallError::Fault => 14,
        }
    }
}