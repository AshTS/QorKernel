use static_assertions::const_assert_eq;

use crate::memory::PAGE_SIZE;

pub const VIRTIO_RING_SIZE_U16: u16 = 1 << 7;
pub const VIRTIO_RING_SIZE: u32 = VIRTIO_RING_SIZE_U16 as u32;
pub const VIRTIO_RING_SIZE_USIZE: usize = VIRTIO_RING_SIZE as usize;

pub const VIRTIO_DESC_F_NEXT: u16 = 1;
pub const VIRTIO_DESC_F_WRITE: u16 = 2;
pub const VIRTIO_DESC_F_INDIRECT: u16 = 4;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Descriptor {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

impl Descriptor {
    /// Construct a new descriptor.
    /// 
    /// # Panics
    /// 
    /// This function will panic if the length cannot fit in 32 bits.
    #[must_use]
    pub fn new(addr: usize, len: usize, flags: u16, next: u16) -> Self {
        Self {
            addr: addr as u64,
            len: len.try_into().unwrap(),
            flags,
            next,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AvailableRing {
    pub flags: u16,
    pub idx: u16,
    pub ring: [u16; VIRTIO_RING_SIZE_USIZE],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UsedElement {
    pub id: u32,
    pub len: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsedRing {
    pub flags: u16,
    pub idx: u16,
    pub ring: [UsedElement; VIRTIO_RING_SIZE_USIZE],
    pub available_event: u16
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Queue {
    pub descriptors: [Descriptor; VIRTIO_RING_SIZE_USIZE],
    pub available: AvailableRing,
    pub index: u16,
    pub acknowledged_used_index: u16,
    pub padding0: [u8; PAGE_SIZE - 4 - core::mem::size_of::<AvailableRing>() - VIRTIO_RING_SIZE_USIZE * core::mem::size_of::<Descriptor>()],
    pub used: UsedRing,
}

const_assert_eq!{(core::mem::size_of::<Queue>() - core::mem::size_of::<UsedRing>()), 4096}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtIOError {
    InvalidMagicValue,
    InvalidDeviceID(u32),
    CouldNotNegotiateFeatures(u32),
    DeviceRejectedFeatures(u32),
    InvalidMaximumQueueSize,
    BadQueueSize
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

impl core::default::Default for AvailableRing {
    fn default() -> Self {
        Self {
            flags: 0,
            idx: 0,
            ring: [0; VIRTIO_RING_SIZE_USIZE]
        }
    }
}

impl core::default::Default for UsedRing {
    fn default() -> Self {
        Self {
            flags: 0,
            idx: 0,
            ring: [UsedElement::default(); VIRTIO_RING_SIZE_USIZE],
            available_event: 0
        }
    }
}

impl Queue {
    pub fn add_descriptor(&mut self, descriptor: Descriptor) -> u16 {
        self.index = (self.index + 1) % VIRTIO_RING_SIZE_U16;
        self.descriptors[self.index as usize] = descriptor;
        if descriptor.flags & VIRTIO_DESC_F_NEXT != 0 {
            self.descriptors[self.index as usize].next = (self.index + 1) % VIRTIO_RING_SIZE_U16;
        };

        self.index
    }
}

impl core::default::Default for Queue {
    fn default() -> Self {
        Self {
            descriptors: [Descriptor::default(); VIRTIO_RING_SIZE_USIZE],
            available: AvailableRing::default(),
            index: 0,
            acknowledged_used_index: 0,
            padding0: [0; PAGE_SIZE - 4 - core::mem::size_of::<AvailableRing>() - VIRTIO_RING_SIZE_USIZE * core::mem::size_of::<Descriptor>()],
            used: UsedRing::default()
        }
    }
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