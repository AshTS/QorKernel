use qor_core::{
    drivers::plic::PLICDriverInterface, interfaces::mmio::MMIOInterface, structures::id::HartID,
};

use super::{raw, InterruptPriority, InterruptSource};

pub struct PLICDriver {
    mmio: MMIOInterface,
}

impl PLICDriver {
    /// # Safety
    ///
    /// This function is unsafe because it takes a raw address and assumes that it is a valid
    /// address for the PLIC.
    #[must_use]
    pub const unsafe fn new(base_address: usize) -> Self {
        Self {
            mmio: MMIOInterface::new(base_address),
        }
    }
}

impl PLICDriverInterface for PLICDriver {
    type PLICDriverError = ();
    type InterruptSource = InterruptSource;
    type Priority = InterruptPriority;

    fn is_initialized(&self) -> bool {
        true
    }

    fn initialize(&self) -> Result<(), Self::PLICDriverError> {
        Ok(())
    }

    fn enable_interrupt_source(
        &self,
        hart_id: HartID,
        source: Self::InterruptSource,
    ) -> Result<(), Self::PLICDriverError> {
        let source_index = source as u32;

        if source_index < 32 {
            unsafe { raw::atomic_interrupt_enable_low_register(&self.mmio, hart_id) }
            .fetch_or(1 << source_index, core::sync::atomic::Ordering::AcqRel);
        }
        else {
            unsafe { raw::atomic_interrupt_enable_high_register(&self.mmio, hart_id) }
            .fetch_or(1 << (source_index - 32), core::sync::atomic::Ordering::AcqRel);
        }
        
        Ok(())
    }

    fn disable_interrupt_source(
        &self,
        hart_id: HartID,
        source: Self::InterruptSource,
    ) -> Result<(), Self::PLICDriverError> {
        let source_index = source as u32;

        if source_index < 32 {
            unsafe { raw::atomic_interrupt_enable_low_register(&self.mmio, hart_id) }
            .fetch_and(!(1 << source_index), core::sync::atomic::Ordering::AcqRel);
        }
        else {
            unsafe { raw::atomic_interrupt_enable_high_register(&self.mmio, hart_id) }
            .fetch_and(!(1 << (source_index - 32)), core::sync::atomic::Ordering::AcqRel);
        }

        Ok(())
    }

    fn set_interrupt_priority(
        &self,
        source: Self::InterruptSource,
        priority: Self::Priority,
    ) -> Result<(), Self::PLICDriverError> {
        unsafe { raw::write_source_priority_register(&self.mmio, source, priority) };
        Ok(())
    }

    fn set_hart_threshold(
        &self,
        hart_id: HartID,
        threshold: Self::Priority,
    ) -> Result<(), Self::PLICDriverError> {
        unsafe { raw::write_threshold_register(&self.mmio, hart_id, threshold) };
        Ok(())
    }

    fn poll_interrupt(
        &self,
        hart_id: HartID,
    ) -> Result<Option<Self::InterruptSource>, Self::PLICDriverError> {
        Ok(InterruptSource::from_num(unsafe {
            raw::read_claim_register(&self.mmio, hart_id)
        }))
    }

    fn complete_interrupt(
        &self,
        hart_id: HartID,
        source: Self::InterruptSource,
    ) -> Result<(), Self::PLICDriverError> {
        unsafe { raw::write_complete_register(&self.mmio, hart_id, source) };
        Ok(())
    }
}
