use core::marker::PhantomData;

use crate::interfaces::bytes::GenericByteInterface;

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

#[allow(clippy::module_name_repetitions)]
pub struct GenericDeviceFileDescriptor<E: core::marker::Sync, Inner:  core::marker::Sync + GenericByteInterface<E>> {
    inner: Inner,
    _phantom: core::marker::PhantomData<E>
}

impl <E: core::marker::Sync, Inner:  core::marker::Sync + GenericByteInterface<E>> GenericDeviceFileDescriptor<E, Inner> {
    pub const fn new(inner: Inner) -> Self {
        Self{
            inner,
            _phantom: PhantomData
        }
    }
}

#[async_trait::async_trait]
impl<E: core::marker::Sync, Inner:  core::marker::Sync + GenericByteInterface<E>> FileDescriptor for GenericDeviceFileDescriptor<E, Inner> {
    /// Read bytes from the file starting at the cursor into `buffer`. Returns the number of bytes read.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation failed.
    async fn read(&self, _buffer: &mut [u8]) -> Result<usize, FileSystemError> {
        todo!()
    }

    /// Writes bytes to the file starting at the cursor. Returns the number of bytes written.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation failed.
    async fn write(&self, buffer: &[u8]) -> Result<usize, FileSystemError> {
        match self.inner.send_bytes(buffer) {
            Ok(()) => Ok(buffer.len()),
            Err(_) => Err(FileSystemError::GenericError)
        }
    }

    /// Seeks to a position in the file.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation failed.
    async fn seek(&self, _seek: SeekMode) -> Result<usize, FileSystemError> {
        Ok(0)
    }
}