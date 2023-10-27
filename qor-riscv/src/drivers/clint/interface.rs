use qor_core::{
    drivers::timer::HardwareTimerDriver,
    interfaces::mmio::MMIOInterface,
    structures::{
        id::HartID,
        time::{Hertz, Microseconds},
    },
};

pub struct HardwareTimer {
    mmio: MMIOInterface,
    step_size: atomic::Atomic<u64>,
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
            step_size: atomic::Atomic::new(1_000_000),
        }
    }

    /// Function which is called every time the timer interrupt is fired.
    pub fn handle_interrupt(&self, hart_id: HartID) {
        let step_size = self.step_size.load(atomic::Ordering::Acquire);
        self.set_time(hart_id, Microseconds(step_size))
            .expect("Unable to set the CLINT Timer rate");
    }

    /// Set the frequency for the timer. Note that this impacts the frequency of the timer on every HART.
    pub fn set_frequency(&self, frequency: Hertz) {
        self.step_size
            .store(1_000_000 / frequency.0, atomic::Ordering::Release);
    }

    /// Start the timer for a given HART
    pub fn start_timer(&self, hart_id: HartID) {
        self.set_time(hart_id, Microseconds(0))
            .expect("Unable to start CLINT timer");
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
