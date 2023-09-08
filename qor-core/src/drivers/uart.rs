use crate::interfaces::bytes::GenericByteInterface;

/// # UART Driver Interface
/// 
/// Exposes the common functionality for all UART Drivers
pub trait UARTDriverInterface: core::fmt::Write + GenericByteInterface<Self::UARTError> {
    type UARTError;

    /// Initialize the UART Driver
    fn is_initialized(&self) -> bool;

    /// Initialize the UART Driver
    /// 
    /// # Errors
    /// 
    /// Returns an error if initialization failed.
    fn initialize(&self) -> Result<(), Self::UARTError>;
}