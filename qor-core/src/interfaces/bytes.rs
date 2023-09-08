/// # Byte Write Interface
/// 
/// Generic interface for sending bytes to a driver
pub trait GenericByteWriteInterface<E> {
    /// Send a byte across the interface
    /// 
    /// # Errors
    /// 
    /// Returns an error if the driver was unable to transmit the byte.
    fn send_byte(&self, byte: u8) -> Result<(), E>;

    /// Send a sequence of bytes stored in a slice across the interface
    /// 
    /// # Errors
    /// 
    /// Returns an error if the driver was unable to transmit any of the bytes being transmitted.
    fn send_bytes(&self, bytes: &[u8]) -> Result<(), E> {
        for b in bytes {
            self.send_byte(*b)?;
        }

        Ok(())
    }
}

/// # Byte Read Interface
/// 
/// Generic interface for reading bytes from a driver
pub trait GenericByteReadInterface<E> {
    /// Read a byte from the interface, returning None if no byte is available
    /// 
    /// # Errors
    /// 
    /// Returns an error if a byte was unable to be read.
    fn read_byte(&self) -> Result<Option<u8>, E>;
}

/// # Generic Byte Interface
/// 
/// Generic interface for sending and receiving bytes to and from a driver
pub trait GenericByteInterface<E> : GenericByteReadInterface<E> + GenericByteWriteInterface<E> {}
