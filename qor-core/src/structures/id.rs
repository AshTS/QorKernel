/// Hardware Thread Identifier
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HartID(pub usize);

impl core::convert::From<usize> for HartID {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
