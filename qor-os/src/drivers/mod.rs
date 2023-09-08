use qor_core::drivers::uart::UARTDriverInterface;
use qor_riscv::drivers::uart::UARTDriver;

// Safety: This is the base address given in the specification for the `virt` platform by QEMU (https://github.com/qemu/qemu/blob/master/hw/riscv/virt.c)
pub static UART_DRIVER: UARTDriver = unsafe { UARTDriver::new(0x1000_0000) };

/// Initialize the UART Driver
/// 
/// # Errors
/// 
/// Returns an error if the driver was unable to be initialized
pub fn initialize_uart_driver() -> Result<(), <UARTDriver as UARTDriverInterface>::UARTError> {

    UART_DRIVER.initialize()
}