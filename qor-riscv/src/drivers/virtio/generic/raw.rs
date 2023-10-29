use qor_core::interfaces::mmio::MMIOInterface;

const MAGIC_VALUE_OFFSET: usize = 0x0000;
const VERSION_VALUE_OFFSET: usize = 0x0004;
const DEVICE_ID_OFFSET: usize = 0x0008;
const VENDOR_ID_OFFSET: usize = 0x000c;
const HOST_FEATURES_OFFSET: usize = 0x0010;
const HOST_FEATURES_SEL_OFFSET: usize = 0x0014;
const GUEST_FEATURES_OFFSET: usize = 0x0020;
const GUEST_FEATURES_SEL_OFFSET: usize = 0x0024;
const GUEST_PAGE_SIZE_OFFSET: usize = 0x0028;
const QUEUE_SEL_OFFSET: usize = 0x0030;
const QUEUE_NUM_MAX_OFFSET: usize = 0x0034;
const QUEUE_NUM_OFFSET: usize = 0x0038;
const QUEUE_ALIGN_OFFSET: usize = 0x003c;
const QUEUE_PFN_OFFSET: usize = 0x0040;
const QUEUE_NOTIFY_OFFSET: usize = 0x0050;
const INTERRUPT_STATUS_OFFSET: usize = 0x0060;
const INTERRUPT_ACK_OFFSET: usize = 0x0064;
const STATUS_OFFSET: usize = 0x0070;
const CONFIG_OFFSET: usize = 0x0100;

use paste::paste;

macro_rules! read_impl {
    ($name: literal) => {
        read_impl!($name, "", "");
    };

    ($name: literal, $extra_docs: literal, $extra_safety: literal) => {
        paste! {
            #[doc="Read the " $name " from the appropriate offset." $extra_docs "\n \n # Safety\n \n The `mmio` interface must point to a valid base address of a memory mapped VirtIO device." $extra_safety]
            #[must_use]
            pub unsafe fn [<read_ $name:snake:lower>](mmio: &MMIOInterface) -> u32 {
                mmio.read_offset([<$name:snake:upper _OFFSET>])
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
            #[doc=" Write the " $name " to the appropriate offset." $extra_docs "\n \n # Safety\n \n The `mmio` interface must point to a valid base address of a memory mapped VirtIO device." $extra_safety]
            pub unsafe fn [<set_ $name:snake:lower>](mmio: &MMIOInterface, data: u32) {
                mmio.write_offset([<$name:snake:upper _OFFSET>], data)
            }
        }
    };
}

macro_rules! atomic_impl {
    ($name: literal) => {
        atomic_impl!($name, "", "");
    };
    ($name: literal, $extra_docs: literal, $extra_safety: literal) => {
        paste! {
            #[doc="Get atomic access to the " $name " in the appropriate offset." $extra_docs "\n \n # Safety\n \n The `mmio` interface must point to a valid base address of a memory mapped VirtIO device." $extra_safety]
            #[must_use]
            pub unsafe fn [<atomic_ $name:snake:lower _register>](mmio: &MMIOInterface) -> &atomic::Atomic<u32> {
                mmio.atomic_access([<$name:snake:upper _OFFSET>])
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

read_impl!("magic_value", "", "");
read_impl!("version_value", "", "");
read_impl!("device_id", "", "");
read_impl!("vendor_id", "", "");
read_impl!("host_features", "", "");
read_impl!("host_features_sel", "", "");
read_write_impl!("guest_features", "", "");
read_impl!("guest_features_sel", "", "");
read_impl!("guest_page_size", "", "");
read_impl!("queue_sel", "", "");
read_impl!("queue_num_max", "", "");
read_impl!("queue_num", "", "");
read_impl!("queue_align", "", "");
read_impl!("queue_pfn", "", "");
read_impl!("queue_notify", "", "");
read_impl!("interrupt_status", "", "");
read_impl!("interrupt_ack", "", "");
read_write_impl!("status", "", "");
atomic_impl!("status", "", "");
read_impl!("config", "", "");