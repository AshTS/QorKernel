/// # Block Device Driver Interface
///
/// Exposes the common functionality for all Block Device Drivers
#[allow(clippy::module_name_repetitions)]
pub trait BlockDeviceDriver<
    const BLOCK_SIZE: usize,
    ReadFuture: core::future::Future<Output = Result<(), Self::BlockDeviceError>>,
    WriteFuture: core::future::Future<Output = Result<(), Self::BlockDeviceError>>,
>
{
    type BlockDeviceError;
    type BlockIndex;

    /// Return true if the Block Device Driver is initialized.
    fn is_initialized(&self) -> bool;

    /// Initialize the Block Device Driver
    ///
    /// # Errors
    ///
    /// Returns an error if initialization failed.
    fn initialize(&self) -> Result<(), Self::BlockDeviceError>;

    /// Read a block from the block device
    fn read_block_range(
        &self,
        index: Self::BlockIndex,
        buffer: &mut [[u8; BLOCK_SIZE]],
    ) -> ReadFuture;

    /// Write a block to the block device
    fn write_block_range(
        &self,
        index: Self::BlockIndex,
        buffer: &[[u8; BLOCK_SIZE]],
    ) -> WriteFuture;

    /// Read a block from the block device
    fn read_block(&self, index: Self::BlockIndex, buffer: &mut [u8; BLOCK_SIZE]) -> ReadFuture {
        self.read_block_range(index, &mut [*buffer])
    }

    /// Write a block to the block device
    fn write_block(&self, index: Self::BlockIndex, buffer: &[u8; BLOCK_SIZE]) -> WriteFuture {
        self.write_block_range(index, &[*buffer])
    }
}
