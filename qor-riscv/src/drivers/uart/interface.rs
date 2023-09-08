use qor_core::{interfaces::{mmio::MMIOInterface, bytes::{GenericByteReadInterface, GenericByteWriteInterface, GenericByteInterface}}, drivers::uart::UARTDriverInterface};

use super::raw;


/// UART Driver for the RISCV Platform
pub struct UARTDriver {
    mmio: MMIOInterface,
    is_initialized: core::sync::atomic::AtomicBool
}

/// Errors which can be returned by the UART Driver API
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UARTError {
    DriverUninitialized
}

impl UARTDriver {
    /// Construct a new UART Driver instance at the given base address.
    /// 
    /// # Safety
    /// 
    /// The `base_address` given must be a valid base address of a memory mapped 16550 UART chipset.  
    #[must_use]
    pub const unsafe fn new(base_address: usize) -> Self {
        Self {
            mmio: MMIOInterface::new(base_address),
            is_initialized: core::sync::atomic::AtomicBool::new(false)
        }
    }

    /// Initialize the UART Connection
    /// 
    /// # Errors
    /// 
    /// Return an error if the initialization was unsuccessful.
    fn inner_initialize(&self) {
        // A value for the Line Control Register which sets the word length to 8 bits. Note that this also sets the DLAB bit to zero.
        let line_control_value = (1 << 0) | (1 << 1);

        // Safety: The requirements on the `mmio` value for the `UARTDriver` ensure this is a valid base address.
        unsafe { raw::set_line_control_register(&self.mmio, line_control_value); }

        // A value for the FIFO Control Register which enables the fifo
        let fifo_control_value = 1 << 0;
        
        // Safety: The requirements on the `mmio` value for the `UARTDriver` ensure this is a valid base address.
        unsafe { raw::set_fifo_control_register(&self.mmio, fifo_control_value); }

        // A value to enable the buffer interrupts
        let interrupt_enable_value = 1 << 0;

        // Safety: The requirements on the `mmio` value for the `UARTDriver` ensure this is a valid base address. Additionally, the Line Control Register currently has the DLAB bit cleared.
        unsafe { raw::set_interrupt_enable_register(&self.mmio, interrupt_enable_value); }

        // Compute the clock divisor
        let divisor: u16 = raw::divisor_from_baud(9600);
        let divisor_low: u8 = (divisor & 0xff) as u8;
        let divisor_high: u8 = (divisor >> 8) as u8;

        // We need to set the data latch bit, so this modified the line control value
        let line_control_value_dlab = line_control_value | (1 << 7);

        // Safety: The requirements on the `mmio` value for the `UARTDriver` ensure this is a valid base address.
        unsafe { raw::set_line_control_register(&self.mmio, line_control_value_dlab); }

        // Safety: The requirements on the `mmio` value for the `UARTDriver` ensure this is a valid base address. Additionally, the Line Control Register currently has the DLAB bit set.
        unsafe { raw::set_divisor_latch_ls_register(&self.mmio, divisor_low) };
        unsafe { raw::set_divisor_latch_ms_register(&self.mmio, divisor_high) };

        // Now that we have set the divisor, we can clear the dlab bit again
        // Safety: The requirements on the `mmio` value for the `UARTDriver` ensure this is a valid base address.
        unsafe { raw::set_line_control_register(&self.mmio, line_control_value); }

        self.is_initialized.store(true, core::sync::atomic::Ordering::Release);
    }

    /// Ensure the driver has been properly initialized.
    ///
    /// # Errors
    ///
    /// This function will return an error if the driver was not initialized.
    fn ensure_initialized(&self) -> Result<(), UARTError> {
        if self.is_initialized.load(core::sync::atomic::Ordering::Acquire) {
            Ok(())
        }
        else {
            Err(UARTError::DriverUninitialized)
        }
    }

    fn inner_read_byte(&self) -> Result<Option<u8>, UARTError> {
        self.ensure_initialized()?;

        // Safety: The requirements on the `mmio` value for the `UARTDriver` ensure this is a valid base address.
        let is_pending_byte = unsafe { raw::read_line_control_register(&self.mmio) } & 1;

        if is_pending_byte == 0 {
            Ok(None)
        }
        else {
            // Safety: The requirements on the `mmio` value for the `UARTDriver` ensure this is a valid base address. The initialization function leaves the DLAB bit cleared, and we have ensured initialization, so we are ready to read data.
            Ok(Some(
                unsafe { raw::read_receiver_buffer_register(&self.mmio) }
            ))
        }
    }

    fn inner_write_byte(&self, b: u8) -> Result<(), UARTError> {
        self.ensure_initialized()?;

        // Safety: The requirements on the `mmio` value for the `UARTDriver` ensure this is a valid base address. The initialization function leaves the DLAB bit cleared, and we have ensured initialization, so we are ready to send data.
        unsafe {
            raw::set_transmitter_holding_register(&self.mmio, b);
        }

        Ok(())
    }
}

impl UARTDriverInterface for UARTDriver {
    type UARTError = UARTError;

    fn is_initialized(&self) -> bool {
        self.is_initialized.load(core::sync::atomic::Ordering::Acquire)
    }

    fn initialize(&self) -> Result<(), Self::UARTError> {
        self.inner_initialize();
        Ok(())
    }
}

impl GenericByteReadInterface<UARTError> for UARTDriver {
    fn read_byte(&self) -> Result<Option<u8>, UARTError> {
        self.inner_read_byte()
    }
}

impl GenericByteWriteInterface<UARTError> for UARTDriver {
    fn send_byte(&self, byte: u8) -> Result<(), UARTError> {
        self.inner_write_byte(byte)
    }
}

impl GenericByteInterface<UARTError> for UARTDriver {}

impl core::fmt::Write for &UARTDriver {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        self.send_bytes(s.as_bytes()).map_err(|_| core::fmt::Error{})
    }
}