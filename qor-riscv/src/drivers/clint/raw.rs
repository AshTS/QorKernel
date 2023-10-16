#![allow(dead_code)]

use qor_core::interfaces::mmio::MMIOInterface;
use qor_core::structures::id::HartID;

const MACHINE_SOFTWARE_INTERRUPT_REGISTER: usize = 0x0;
const MACHINE_TIME_COMPARE_REGISTER: usize = 0x4000;
const MACHINE_TIME_REGISTER: usize = 0xBFF8;

use paste::paste;

macro_rules! read_impl {
    ($name: literal, $size: ty) => {
        read_impl!($name, $size, "", "");
    };

    ($name: literal, $size: ty, $extra_docs: literal, $extra_safety: literal) => {
        paste! {
            #[doc="Read the " $name " register for the given HART ID." $extra_docs "\n \n # Safety\n \n The `mmio` interface must point to a valid base address of a memory mapped CLINT device." $extra_safety]
            pub unsafe fn [<read_ $name:snake:lower _register>](mmio: &MMIOInterface, hart_id: HartID) -> $size {
                mmio.read_offset([<$name:snake:upper _REGISTER>] + hart_id.0 * core::mem::size_of::<$size>())
            }
        }
    };
}

macro_rules! write_impl {
    ($name: literal, $size: ty) => {
        write_impl!($name, $size, "", "");
    };

    ($name: literal, $size: ty, $extra_docs: literal, $extra_safety: literal) => {
        paste! {
            #[doc=" Write to the " $name " register for the given HART ID." $extra_docs "\n \n # Safety\n \n The `mmio` interface must point to a valid base address of a memory mapped CLINT device." $extra_safety]
            pub unsafe fn [<set_ $name:snake:lower _register>](mmio: &MMIOInterface, data: $size, hart_id: HartID) {
                mmio.write_offset([<$name:snake:upper _REGISTER>] + hart_id.0 * core::mem::size_of::<$size>(), data)
            }
        }
    };
}

macro_rules! read_write_impl {
    ($name: literal, $size: ty) => {
        read_impl!($name, $size);
        write_impl!($name, $size);
    };
    ($name: literal, $size: ty, $extra_docs: literal, $extra_safety: literal) => {
        read_impl!($name, $size, $extra_docs, $extra_safety);
        write_impl!($name, $size, $extra_docs, $extra_safety);
    };
}

read_write_impl!("machine_software_interrupt", u32);
read_write_impl!("machine_time_compare", u64);
read_write_impl!("machine_time", u64);
