use qor_core::drivers::uart::UARTDriverInterface;
use qor_riscv::drivers::{clint::HardwareTimer, plic::PLICDriver, uart::UARTDriver};

pub mod interrupts;
pub use interrupts::*;

use self::virtio::block::VirtIOBlockDeviceError;

pub mod virtio;

// Safety: This is the base address given in the specification for the `virt` platform by QEMU (https://github.com/qemu/qemu/blob/master/hw/riscv/virt.c)
pub static UART_DRIVER: UARTDriver = unsafe { UARTDriver::new(0x1000_0000) };

// Safety: This is the base address given in the specification for the `virt` platform by QEMU (https://github.com/qemu/qemu/blob/master/hw/riscv/virt.c)
pub static CLINT_DRIVER: HardwareTimer = unsafe { HardwareTimer::new(0x200_0000) };

// Safety: This is the base address given in the specification for the `virt` platform by QEMU (https://github.com/qemu/qemu/blob/master/hw/riscv/virt.c)
pub static PLIC_DRIVER: PLICDriver = unsafe { PLICDriver::new(0xc00_0000) };

pub static BLOCK_DRIVER: atomic_ref::AtomicRef<
    'static,
    alloc::boxed::Box<
        (dyn qor_core::drivers::block::BlockDeviceDriver<512, VirtIOBlockDeviceError, u32>
             + Send
             + Sync),
    >,
> = atomic_ref::AtomicRef::new(None);

/// Initialize the UART Driver
///
/// # Errors
///
/// Returns an error if the driver was unable to be initialized
pub fn initialize_uart_driver() -> Result<(), <UARTDriver as UARTDriverInterface>::UARTError> {
    UART_DRIVER.initialize()
}

/// Access the block driver
pub fn get_block_driver() -> &'static alloc::boxed::Box<
    (dyn qor_core::drivers::block::BlockDeviceDriver<512, VirtIOBlockDeviceError, u32>
         + Send
         + Sync),
> {
    BLOCK_DRIVER
        .load(core::sync::atomic::Ordering::Acquire)
        .expect("Block device driver not initialized")
}
