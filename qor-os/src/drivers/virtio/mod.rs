use qor_riscv::drivers::virtio::generic::structures::DeviceID;

use block::VirtIOBlockDevice;

pub mod block;

/// Prove the Virt IO Address Range for Devices
pub fn probe_virt_io_address_range() {
    for index in 0..8 {
        let address = 0x1000_8000 - (index * 0x1000);

        if let Ok(virt_io) = unsafe { qor_riscv::drivers::virtio::probe_virt_io_address(address) } {
            if let Ok(Some(device_id)) = virt_io.verify() {
                if device_id == DeviceID::BlockDevice {
                    info!("Initializing Block Device");
                    virt_io
                        .start_setup(|v| Some(v & !(1 << 5)))
                        .expect("Setup Failed");
                    let mut block = VirtIOBlockDevice::new(virt_io);
                    block
                        .initialize()
                        .expect("Unable to initialize block device");

                    let block_inner =
                        alloc::boxed::Box::new(block::interface::BlockDriver::new(block));

                    let block = alloc::boxed::Box::leak(alloc::boxed::Box::new(
                        block_inner as alloc::boxed::Box<dyn qor_core::drivers::block::BlockDeviceDriver<512, crate::drivers::virtio::block::driver::VirtIOBlockDeviceError, u32> + core::marker::Send + core::marker::Sync>
                    ));

                    crate::drivers::BLOCK_DRIVER
                        .store(Some(block), core::sync::atomic::Ordering::Release);
                    info!("Block Device Initialization Complete");
                }
            }
        }
    }
}
