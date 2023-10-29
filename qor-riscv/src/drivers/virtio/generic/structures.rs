
#[repr(C)]
pub struct Descriptor {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C)]
pub struct AvailableRing {
    flags: u16,
    idx: u16,
    ring: [u16; 1024],
}

#[repr(C)]
pub struct UsedElement {
    id: u32,
    len: u32,
}

#[repr(C)]
pub struct UsedRing {
    flags: u16,
    idx: u16,
    ring: [UsedElement; 1024],
    available_event: u16
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtIOError {
    InvalidMagicValue,
    InvalidDeviceID(u32),
    CouldNotNegotiateFeatures(u32),
    DeviceRejectedFeatures(u32)
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceID {
    NetworkCard = 1,
    BlockDevice = 2,
    Console = 3,
    EntropySource = 4,
    MemoryBalloon = 5,
    IOMemory = 6,
    SignalDistribution = 9,
    GPU = 16,
    InputDevice=18,
}

impl core::convert::TryFrom<u32> for DeviceID {
    type Error = VirtIOError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::NetworkCard),
            2 => Ok(Self::BlockDevice),
            3 => Ok(Self::Console),
            4 => Ok(Self::EntropySource),
            5 => Ok(Self::MemoryBalloon),
            6 => Ok(Self::IOMemory),
            9 => Ok(Self::SignalDistribution),
            16 => Ok(Self::GPU),
            18 => Ok(Self::InputDevice),
            _ => Err(VirtIOError::InvalidDeviceID(value)),
        }
    }
}