#![allow(dead_code)]

use super::InterruptPriority;
use super::InterruptSource;
use qor_core::interfaces::mmio::MMIOInterface;
use qor_core::structures::id::HartID;

pub const SOURCE_PRIORITY_REGISTER_BASE: usize = 0x0;
pub const INTERRUPT_ENABLE_REGISTER_BASE: usize = 0x2000;
pub const THRESHOLD_REGISTER_BASE: usize = 0x20_0000;
pub const CLAIM_REGISTER_BASE: usize = 0x20_0004;
pub const COMPLETE_REGISTER_BASE: usize = 0x20_0004;

use paste::paste;

macro_rules! read_impl_source_index {
    ($name: literal, $extra_docs: literal, $extra_safety: literal, $size: ty) => {
        read_impl_source_index!($name, $extra_docs, $extra_safety, $size, core::mem::size_of::<$size>());
    };
    ($name: literal, $extra_docs: literal, $extra_safety: literal, $size: ty, $offset: expr) => {
        paste! {
            #[doc="Read the " $name " register for the given interrupt source." $extra_docs "\n \n # Safety\n \n The `mmio` interface must point to a valid base address of a memory mapped FU540-C000 PLIC  device." $extra_safety]
            pub unsafe fn [<read_ $name:snake:lower _register>](mmio: &MMIOInterface, offset: InterruptSource) -> $size {
                mmio.read_offset([<$name:snake:upper _REGISTER_BASE>] + offset as usize * $offset)
            }
        }
    };
}

macro_rules! write_impl_source_index {
    ($name: literal, $extra_docs: literal, $extra_safety: literal, $size: ty) => {
        write_impl_source_index!($name, $extra_docs, $extra_safety, $size, core::mem::size_of::<$size>());
    };
    ($name: literal, $extra_docs: literal, $extra_safety: literal, $size: ty, $offset: expr) => {
        paste! {
            #[doc="Write to the " $name " register for the given interrupt source." $extra_docs "\n \n # Safety\n \n The `mmio` interface must point to a valid base address of a memory mapped FU540-C000 PLIC  device." $extra_safety]
            pub unsafe fn [<write_ $name:snake:lower _register>](mmio: &MMIOInterface, offset: InterruptSource, data: $size) {
                mmio.write_offset([<$name:snake:upper _REGISTER_BASE>] + offset as usize * $offset, data);
            }
        }
    };
}

macro_rules! atomic_impl_source_index {
    ($name: literal, $extra_docs: literal, $extra_safety: literal, $size: ty) => {
        read_impl_source_index!($name, $extra_docs, $extra_safety, $size, core::mem::size_of::<$size>());
    };
    ($name: literal, $extra_docs: literal, $extra_safety: literal, $size: ty, $offset: expr) => {
        paste! {
            #[doc="Get atomic access to the " $name " register for the given interrupt source." $extra_docs "\n \n # Safety\n \n The `mmio` interface must point to a valid base address of a memory mapped FU540-C000 PLIC  device." $extra_safety]
            pub unsafe fn [<atomic_ $name:snake:lower _register>](mmio: &MMIOInterface, offset: InterruptSource) -> &atomic::Atomic<$size> {
                mmio.atomic_access([<$name:snake:upper _REGISTER_BASE>] + offset as usize * $offset)
            }
        }
    };
}

macro_rules! read_impl_hart_index {
    ($name: literal, $extra_docs: literal, $extra_safety: literal, $size: ty) => {
        read_impl_hart_index!($name, $extra_docs, $extra_safety, $size, core::mem::size_of::<$size>());
    };
    ($name: literal, $extra_docs: literal, $extra_safety: literal, $size: ty, $offset: expr) => {
        paste! {
            #[doc="Read the " $name " register for the given HART." $extra_docs "\n \n # Safety\n \n The `mmio` interface must point to a valid base address of a memory mapped FU540-C000 PLIC  device." $extra_safety]
            pub unsafe fn [<read_ $name:snake:lower _register>](mmio: &MMIOInterface, offset: HartID) -> $size {
                mmio.read_offset([<$name:snake:upper _REGISTER_BASE>] + offset.0 as usize * $offset)
            }
        }
    };
}

macro_rules! write_impl_hart_index {
    ($name: literal, $extra_docs: literal, $extra_safety: literal, $size: ty) => {
        write_impl_hart_index!($name, $extra_docs, $extra_safety, $size, core::mem::size_of::<$size>());
    };
    ($name: literal, $extra_docs: literal, $extra_safety: literal, $size: ty, $offset: expr) => {
        paste! {
            #[doc="Write to the " $name " register for the given HART." $extra_docs "\n \n # Safety\n \n The `mmio` interface must point to a valid base address of a memory mapped FU540-C000 PLIC  device." $extra_safety]
            pub unsafe fn [<write_ $name:snake:lower _register>](mmio: &MMIOInterface, offset: HartID, data: $size) {
                mmio.write_offset([<$name:snake:upper _REGISTER_BASE>] + offset.0 as usize * $offset, data);
            }
        }
    };
}

macro_rules! read_write_impl_source_index {
    ($name: literal, $extra_docs: literal, $extra_safety: literal, $size: ty, $offset: expr) => {
        read_impl_source_index!($name, $extra_docs, $extra_safety, $size, $offset);
        write_impl_source_index!($name, $extra_docs, $extra_safety, $size, $offset);
    };
}

macro_rules! read_write_impl_hart_index {
    ($name: literal, $extra_docs: literal, $extra_safety: literal, $size: ty, $offset: expr) => {
        read_impl_hart_index!($name, $extra_docs, $extra_safety, $size, $offset);
        write_impl_hart_index!($name, $extra_docs, $extra_safety, $size, $offset);
    };
}

read_write_impl_source_index!("source_priority", "", "", InterruptPriority, 4);
read_write_impl_hart_index!("interrupt_enable", "", "", u64, 0x80);
atomic_impl_source_index!("interrupt_enable", "", "", u64, 0x80);
read_write_impl_hart_index!("threshold", "", "", InterruptPriority, 0x1000);
read_impl_hart_index!("claim", "", "", u32, 0x1000);
write_impl_hart_index!("complete", "", "", InterruptSource, 0x1000);
