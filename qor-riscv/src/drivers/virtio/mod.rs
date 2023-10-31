use self::generic::{driver::VirtIOWrapper, structures::VirtIOError};

pub mod generic;

/// This function probes the Virt IO device at a given address.
///
/// # Errors
///
/// This function will return an error if the device is invalid, or unable to be initialized.
///
/// # Safety
///
/// The `base` address must be a valid base address for a memory mapped Virt IO device.
pub unsafe fn probe_virt_io_address(base: usize) -> Result<VirtIOWrapper, VirtIOError> {
    let virtio = VirtIOWrapper::new(base);
    virtio.verify()?;

    Ok(virtio)
}
