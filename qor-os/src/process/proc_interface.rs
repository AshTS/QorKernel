use core::borrow::Borrow;

use alloc::{collections::BTreeMap, boxed::Box, sync::Arc};
use qor_core::interfaces::{fs::{FileDescriptor, FileSystemError, SeekMode}, bytes::GenericByteWriteInterface};

use crate::drivers::UART_DRIVER;

pub struct ProcessData {
    pub file_descriptors: BTreeMap<usize, Arc<dyn FileDescriptor>>
}

pub struct UARTFileDescriptor {}

#[async_trait::async_trait]
impl FileDescriptor for UARTFileDescriptor {
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
        match UART_DRIVER.borrow().send_bytes(buffer) {
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

impl ProcessData {
    pub fn new() -> Self {
        // TODO: Don't immediately just add these, eventually a process will be opening these
        let mut file_descriptors = BTreeMap::new();

        file_descriptors.insert(0, Arc::new(UARTFileDescriptor {}) as Arc<dyn FileDescriptor>);
        file_descriptors.insert(1, Arc::new(UARTFileDescriptor {}) as Arc<dyn FileDescriptor>);

        Self {
            file_descriptors
        }
    }
}