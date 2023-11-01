use qor_core::{drivers::block::BlockDeviceDriver, sync::Mutex};

use alloc::boxed::Box;

use super::{VirtIOBlockDevice, VirtIOBlockDeviceError};

pub struct BlockDriver(Mutex<VirtIOBlockDevice>);

impl BlockDriver {
    /// Creates a new [`BlockDriver`] by wrapping a [`VirtIOBlockDevice`] in a [`Mutex`].
    pub const fn new(block: VirtIOBlockDevice) -> Self {
        Self(Mutex::new(block))
    }
}

#[async_trait::async_trait]
impl BlockDeviceDriver<512, VirtIOBlockDeviceError, u32> for BlockDriver {
    fn is_initialized(&self) -> bool {
        true
    }

    fn initialize(&self) -> Result<(), VirtIOBlockDeviceError> {
        Ok(())
    }

    /// Read a block from the block device
    async fn read_blocks<'b, 'a: 'b>(
        &'b self,
        index: u32,
        buffer: &'a mut [[u8; 512]],
    ) -> Result<(), VirtIOBlockDeviceError> {
        let mut guard = self.0.async_lock().await;
        guard.non_blocking_read(buffer, index as usize).await
    }

    /// Write a block to the block device
    async fn write_blocks<'b, 'a: 'b>(
        &'b self,
        index: u32,
        buffer: &'a [[u8; 512]],
    ) -> Result<(), VirtIOBlockDeviceError> {
        let mut guard = self.0.async_lock().await;
        guard.non_blocking_write(buffer, index as usize).await
    }
}
