use qor_core::{
    drivers::timer::HardwareTimerDriver,
    interfaces::mmio::MMIOInterface,
    structures::{id::HartID, time::Microseconds},
};

pub struct HardwareTimer {
    mmio: MMIOInterface,
}

impl HardwareTimer {
    /// Construct a new Hardware Timer Driver instance at the given base address.
    ///
    /// # Safety
    ///
    /// The `base_address` given must be a valid base address of a memory mapped CLINT device.  
    #[must_use]
    pub const unsafe fn new(base_address: usize) -> Self {
        Self {
            mmio: MMIOInterface::new(base_address),
        }
    }
}

impl HardwareTimerDriver for HardwareTimer {
    type HardwareTimerError = ();

    fn is_initialized(&self) -> bool {
        true
    }

    fn initialize(&self) -> Result<(), Self::HardwareTimerError> {
        Ok(())
    }

    fn set_time(&self, id: HartID, time: Microseconds) -> Result<(), Self::HardwareTimerError> {
        let current_time = self.time(id)?;

        // Safety: The requirements on the `mmio` value for the `HardwareTimer` ensure this is a valid base address.
        unsafe {
            super::raw::set_machine_time_compare_register(
                &self.mmio,
                10 * (current_time.0 + time.0),
                id,
            );
        }

        Ok(())
    }

    fn time(&self, id: HartID) -> Result<Microseconds, Self::HardwareTimerError> {
        // Safety: The requirements on the `mmio` value for the `HardwareTimer` ensure this is a valid base address.
        let raw_time = unsafe { super::raw::read_machine_time_register(&self.mmio, id) };

        Ok(Microseconds(raw_time / 10))
    }

    fn reset(&self, id: HartID) -> Result<(), Self::HardwareTimerError> {
        // Safety: The requirements on the `mmio` value for the `HardwareTimer` ensure this is a valid base address.
        unsafe {
            super::raw::set_machine_time_register(&self.mmio, 0, id);
        }

        Ok(())
    }
}
