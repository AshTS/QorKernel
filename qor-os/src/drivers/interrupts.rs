use qor_core::structures::id::HartID;
use qor_riscv::drivers::plic::InterruptSource;

pub const UART_INTERRUPT: InterruptSource = InterruptSource::Source10;

pub const VIRTIO_INTERRUPT_1: InterruptSource = InterruptSource::Source1;
pub const VIRTIO_INTERRUPT_2: InterruptSource = InterruptSource::Source2;
pub const VIRTIO_INTERRUPT_3: InterruptSource = InterruptSource::Source3;
pub const VIRTIO_INTERRUPT_4: InterruptSource = InterruptSource::Source4;
pub const VIRTIO_INTERRUPT_5: InterruptSource = InterruptSource::Source5;
pub const VIRTIO_INTERRUPT_6: InterruptSource = InterruptSource::Source6;
pub const VIRTIO_INTERRUPT_7: InterruptSource = InterruptSource::Source7;
pub const VIRTIO_INTERRUPT_8: InterruptSource = InterruptSource::Source8;

pub const VIRTIO_INTERRUPTS: [InterruptSource; 8] = [
    VIRTIO_INTERRUPT_1,
    VIRTIO_INTERRUPT_2,
    VIRTIO_INTERRUPT_3,
    VIRTIO_INTERRUPT_4,
    VIRTIO_INTERRUPT_5,
    VIRTIO_INTERRUPT_6,
    VIRTIO_INTERRUPT_7,
    VIRTIO_INTERRUPT_8,
];

/// Initialize the PLIC for the boot HART
pub fn initialize_plic(boot_hart: HartID) {
    use crate::qor_core::drivers::plic::PLICDriverInterface;

    let plic = &crate::drivers::PLIC_DRIVER;
    plic.initialize().expect("Unable to initialize PLIC");
    for int in VIRTIO_INTERRUPTS {
        plic.set_interrupt_priority(int, qor_riscv::drivers::plic::InterruptPriority::Priority7)
            .expect("Unable to set VirtIO interrupt priority");

        plic.enable_interrupt_source(boot_hart, int)
            .expect("Unable to enable VirtIO interrupts");
    }

    plic.set_interrupt_priority(
        UART_INTERRUPT,
        qor_riscv::drivers::plic::InterruptPriority::Priority7,
    )
    .expect("Unable to set UART interrupt priority");

    plic.enable_interrupt_source(boot_hart, UART_INTERRUPT)
        .expect("Unable to enable UART interrupts");

    plic.set_hart_threshold(
        boot_hart,
        qor_riscv::drivers::plic::InterruptPriority::Priority1,
    )
    .expect("Unable to set PLIC threshold");
}
