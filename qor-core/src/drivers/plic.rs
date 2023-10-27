use crate::structures::id::HartID;

/// # PLIC Interface
///
/// Exposes the common functionality for all PLIC Drivers
pub trait PLICDriverInterface {
    type PLICDriverError;
    type InterruptSource;
    type Priority;

    /// Return true if the driver is initialized.
    fn is_initialized(&self) -> bool;

    /// Initialize the PLIC Driver.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization failed.
    fn initialize(&self) -> Result<(), Self::PLICDriverError>;

    /// Enable a specific interrupt source for a particular hart.
    ///
    /// # Errors
    ///
    /// Returns an error if the interrupt source could not be enabled.
    fn enable_interrupt_source(
        &self,
        hart_id: HartID,
        source: Self::InterruptSource,
    ) -> Result<(), Self::PLICDriverError>;

    /// Disable a specific interrupt source for a particular hart.
    ///
    /// # Errors
    ///
    /// Returns an error if the interrupt source could not be enabled.
    fn disable_interrupt_source(
        &self,
        hart_id: HartID,
        source: Self::InterruptSource,
    ) -> Result<(), Self::PLICDriverError>;

    /// Set the priority of a specific interrupt source.
    ///
    /// # Errors
    ///
    /// Returns an error if the interrupt source priority could not be set.
    fn set_interrupt_priority(
        &self,
        source: Self::InterruptSource,
        priority: Self::Priority,
    ) -> Result<(), Self::PLICDriverError>;

    /// Set the interrupt threshold of a specific hart.
    ///
    /// # Errors
    ///
    /// Returns an error if the interrupt threshold could not be set.
    fn set_hart_threshold(
        &self,
        hart_id: HartID,
        threshold: Self::Priority,
    ) -> Result<(), Self::PLICDriverError>;

    /// Poll the interrupt that is currently pending for a specific hart.
    ///
    /// # Errors
    ///
    /// Returns an error if the interrupt could not be polled.
    fn poll_interrupt(
        &self,
        hart_id: HartID,
    ) -> Result<Option<Self::InterruptSource>, Self::PLICDriverError>;

    /// Complete the interrupt that is currently pending for a specific hart.
    ///
    /// # Errors
    ///
    /// Returns an error if the interrupt could not be completed.
    fn complete_interrupt(
        &self,
        hart_id: HartID,
        source: Self::InterruptSource,
    ) -> Result<(), Self::PLICDriverError>;
}
