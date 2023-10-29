use qor_core::interfaces::mmio::MMIOInterface;
use super::{raw, structures::{DeviceID, VirtIOError}, bits};

/// Wrapper object for a Virt IO device
pub struct VirtIOWrapper {
    mmio_layer: MMIOInterface
}

impl VirtIOWrapper {
    /// Construct a new `VirtIOWrapper` at a base address. Note that this constructor will not do any error handling.
    /// 
    /// # Safety
    /// 
    /// The `base` address must be a valid base address for a Virt IO device.
    #[must_use]
    pub const unsafe fn new(base: usize) -> Self {
        Self {
            mmio_layer: MMIOInterface::new(base)
        }
    }

    /// Verify that this is a valid Virt IO device.
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the device is invalid.
    pub fn verify(&self) -> Result<Option<DeviceID>, VirtIOError> {
        self.check_magic()?;
        self.device_id()
    }

    /// Verify the magic value of the Virt IO device.
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the magic value is incorrect.
    pub fn check_magic(&self) -> Result<(), VirtIOError> {
        // Safety: The only safe way to construct a `VirtIOWrapper` is by providing a proper base address.
        if unsafe { raw::read_magic_value(&self.mmio_layer) } == 0x7472_6976 {
            Ok(())
        }
        else {
            Err(VirtIOError::InvalidMagicValue)
        }
    }

    /// Get the device id of this Virt IO device.
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the device id is invalid.
    pub fn device_id(&self) -> Result<Option<DeviceID>, VirtIOError> {
        // Safety: The only safe way to construct a `VirtIOWrapper` is by providing a proper base address.
        let value = unsafe { raw::read_device_id(&self.mmio_layer) };
        if value == 0 {
            Ok(None)
        }
        else {
            value.try_into().map(Some)
        }
    }

    /// Reset the device.
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the device could not be reset.
    pub fn reset(&self) -> Result<(), VirtIOError> {
        // Safety: The only safe way to construct a `VirtIOWrapper` is by providing a proper base address.
        unsafe { raw::set_status(&self.mmio_layer, 0) };
        Ok(())
    }

    /// Set a set of bits in the status register.
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the given bits in status could not be set.
    pub fn set_status_bits(&self, mask: u32) -> Result<(), VirtIOError> {
        // Safety: The only safe way to construct a `VirtIOWrapper` is by providing a proper base address.
        unsafe { raw::atomic_status_register(&self.mmio_layer) }.fetch_or(mask, core::sync::atomic::Ordering::AcqRel);
        Ok(())
    }

    /// Clear a set of bits in the status register.
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the given bits in status could not be cleared.
    pub fn clear_status_bits(&self, mask: u32) -> Result<(), VirtIOError> {
        // Safety: The only safe way to construct a `VirtIOWrapper` is by providing a proper base address.
        unsafe { raw::atomic_status_register(&self.mmio_layer) }.fetch_and(!mask, core::sync::atomic::Ordering::AcqRel);
        Ok(())
    }

    /// Start device setup
    ///
    /// # Errors
    /// 
    /// This function will return an error if the device configuration could not be started.
    pub fn start_setup(&self, negotiation: impl Fn(u32)->Option<u32>) -> Result<(), VirtIOError> {
        // Set the acknowledge bit
        self.set_status_bits(bits::STATUS_BIT_ACKNOWLEDGE)?;
        
        // Set the driver bit
        self.set_status_bits(bits::STATUS_BIT_DRIVER)?;
        
        // Read the device features
        let device_features = unsafe { raw::read_host_features(&self.mmio_layer) };

        // Negotiate the device features
        let Some(negotiated_features) = negotiation(device_features) else {
            return Err(VirtIOError::CouldNotNegotiateFeatures(device_features))
        };

        // Inform the device of the features we accept
        unsafe { raw::set_guest_features(&self.mmio_layer, negotiated_features) };

        // Set the features ok bit
        self.set_status_bits(bits::STATUS_BIT_FEATURES_OK)?;

        // Read the status register back, so we can verify the features ok bit is still set.
        let status = unsafe { raw::read_status(&self.mmio_layer) };
        if status & bits::STATUS_BIT_FEATURES_OK == 0 {
            return Err(VirtIOError::DeviceRejectedFeatures(negotiated_features));
        }

        Ok(())
    }

    /// Complete device setup
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the device configuration could not be completed.
    pub fn complete_setup(&self) -> Result<(), VirtIOError> {
        self.set_status_bits(bits::STATUS_BIT_DRIVER_OK)?;
        Ok(())
    }
} 