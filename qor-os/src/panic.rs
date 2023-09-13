#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    error!("{}", info);
    loop {
        core::hint::spin_loop();
    }
}
