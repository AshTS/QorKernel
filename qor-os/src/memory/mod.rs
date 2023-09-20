pub mod bitmap;
pub use bitmap::*;

pub mod bump;
pub use bump::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocatorInitializationError;
