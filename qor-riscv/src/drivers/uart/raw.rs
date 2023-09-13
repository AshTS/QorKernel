#![allow(dead_code)]

use qor_core::interfaces::mmio::MMIOInterface;

// Registers which are always available
const INTERRUPT_IDENTITY_REGISTER: usize = 0x02;
const FIFO_CONTROL_REGISTER: usize = 0x02;
const LINE_CONTROL_REGISTER: usize = 0x03;
const MODEM_CONTROL_REGISTER: usize = 0x04;
const LINE_STATUS_REGISTER: usize = 0x05;
const MODEM_STATUS_REGISTER: usize = 0x06;
const SCRATCH_REGISTER: usize = 0x07;

// To use these, the DLAB Bit (Bit 7 of the Line Control Register) must be cleared
const RECEIVER_BUFFER_REGISTER: usize = 0x00;
const TRANSMITTER_HOLDING_REGISTER: usize = 0x00;
const INTERRUPT_ENABLE_REGISTER: usize = 0x01;

// To use these, the DLAB Bit (Bit 7 of the Line Control Register) must be set
const DIVISOR_LATCH_LS_REGISTER: usize = 0x00;
const DIVISOR_LATCH_MS_REGISTER: usize = 0x01;

use paste::paste;

macro_rules! read_impl {
    ($name: literal) => {
        read_impl!($name, "", "");
    };

    ($name: literal, $extra_docs: literal, $extra_safety: literal) => {
        paste! {
            #[doc="Read the " $name " register." $extra_docs "\n \n # Safety\n \n The `mmio` interface must point to a valid base address of a memory mapped NS16550a device." $extra_safety]
            pub unsafe fn [<read_ $name:snake:lower _register>](mmio: &MMIOInterface) -> u8 {
                mmio.read_offset([<$name:snake:upper _REGISTER>])
            }
        }
    };
}

macro_rules! write_impl {
    ($name: literal) => {
        write_impl!($name, "", "");
    };

    ($name: literal, $extra_docs: literal, $extra_safety: literal) => {
        paste! {
            #[doc=" Write to the " $name " register." $extra_docs "\n \n # Safety\n \n The `mmio` interface must point to a valid base address of a memory mapped NS16550a device." $extra_safety]
            pub unsafe fn [<set_ $name:snake:lower _register>](mmio: &MMIOInterface, byte: u8) {
                mmio.write_offset([<$name:snake:upper _REGISTER>], byte)
            }
        }
    };
}

macro_rules! read_write_impl {
    ($name: literal) => {
        read_impl!($name);
        write_impl!($name);
    };
    ($name: literal, $extra_docs: literal, $extra_safety: literal) => {
        read_impl!($name, $extra_docs, $extra_safety);
        write_impl!($name, $extra_docs, $extra_safety);
    };
}

read_impl!("interrupt_identity");
write_impl!("fifo_control");

read_write_impl!("line_control");
read_write_impl!("modem_control");
read_write_impl!("line_status");
read_write_impl!("modem_status");
read_write_impl!("scratch");

read_impl!(
    "receiver_buffer",
    "",
    "Additionally, the DLAB (the 7th bit of the Line Control Register) bit must be clear."
);
write_impl!(
    "transmitter_holding",
    "",
    "Additionally, the DLAB (the 7th bit of the Line Control Register) bit must be clear."
);
write_impl!(
    "interrupt_enable",
    "",
    "Additionally, the DLAB (the 7th bit of the Line Control Register) bit must be clear."
);

read_write_impl!(
    "divisor_latch_ls",
    "",
    "Additionally, the DLAB (the 7th bit of the Line Control Register) bit must be set."
);
read_write_impl!(
    "divisor_latch_ms",
    "",
    "Additionally, the DLAB (the 7th bit of the Line Control Register) bit must be set."
);

pub const fn divisor_from_baud(baud: u32) -> u16 {
    let baud16 = baud * 16;
    (((22_729_000 + baud16 - 1) / baud16) & 0xffff) as u16
}
