use alloc::boxed::Box;

/// # Block Device Driver Interface
///
/// Exposes the common functionality for all Block Device Drivers
#[allow(clippy::module_name_repetitions)]
#[async_trait::async_trait]
pub trait BlockDeviceDriver<
    const BLOCK_SIZE: usize,
    BlockDeviceError: core::fmt::Debug + Send + Sync,
    BlockIndex: Copy + Send + Sync,
>
{
    /// Return true if the Block Device Driver is initialized.
    fn is_initialized(&self) -> bool;

    /// Initialize the Block Device Driver
    ///
    /// # Errors
    ///
    /// Returns an error if initialization failed.
    fn initialize(&self) -> Result<(), BlockDeviceError>;

    /// Read a block from the block device
    async fn read_blocks<'b, 'a: 'b>(
        &'b self,
        index: BlockIndex,
        buffer: &'a mut [[u8; BLOCK_SIZE]],
    ) -> Result<(), BlockDeviceError>;

    /// Write a block to the block device
    async fn write_blocks<'b, 'a: 'b>(
        &'b self,
        index: BlockIndex,
        buffer: &'a [[u8; BLOCK_SIZE]],
    ) -> Result<(), BlockDeviceError>;
}
