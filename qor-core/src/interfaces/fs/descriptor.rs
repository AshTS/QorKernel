use super::FileSystemError;

use alloc::boxed::Box;

pub enum SeekMode {
    Set(usize),
    End(isize),
    Current(isize),
}

#[allow(clippy::module_name_repetitions)]
#[async_trait::async_trait]
pub trait FileDescriptor {
    /// Read bytes from the file starting at the cursor into `buffer`. Returns the number of bytes read.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation failed.
    async fn read(&self, buffer: &mut [u8]) -> Result<usize, FileSystemError>;

    /// Writes bytes to the file starting at the cursor. Returns the number of bytes written.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation failed.
    async fn write(&self, buffer: &[u8]) -> Result<usize, FileSystemError>;

    /// Seeks to a position in the file.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation failed.
    async fn seek(&self, seek: SeekMode) -> Result<usize, FileSystemError>;
}
