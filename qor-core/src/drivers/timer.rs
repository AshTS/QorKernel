use crate::structures::{
    id::HartID,
    time::{Hertz, Microseconds},
};

/// # UART Driver Interface
///
/// Exposes the common functionality for all Hardware Timer Drivers
pub trait HardwareTimerDriver {
    type HardwareTimerError;

    /// Return true if the Hardware Timer Driver is initialized
    fn is_initialized(&self) -> bool;

    /// Initialize the Hardware Timer Driver
    ///
    /// # Errors
    ///
    /// Returns an error if initialization failed.
    fn initialize(&self) -> Result<(), Self::HardwareTimerError>;

    /// Set the time until the next tick in microseconds.
    ///
    /// # Errors
    ///
    /// Returns an error if the time to next tick could not be set.
    fn set_time(&self, id: HartID, time: Microseconds) -> Result<(), Self::HardwareTimerError>;

    /// Set the time until the next tick based on a frequency.
    ///
    /// # Errors
    ///
    /// Returns an error if the time to next tick could not be set.
    fn set_time_rate(&self, id: HartID, frequency: Hertz) -> Result<(), Self::HardwareTimerError> {
        let time = Microseconds(1_000_000 / frequency.0);
        self.set_time(id, time)
    }

    /// Get the current time since last reset.
    ///
    /// # Errors
    ///
    /// Returns an error if the time could not be read.
    fn time(&self, id: HartID) -> Result<Microseconds, Self::HardwareTimerError>;

    /// Reset the hardware timer.
    ///
    /// # Errors
    ///
    /// Returns an error if the timer could not be reset.
    fn reset(&self, id: HartID) -> Result<(), Self::HardwareTimerError>;
}
