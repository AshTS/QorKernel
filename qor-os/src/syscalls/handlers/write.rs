use qor_core::{structures::syscall_error::SyscallError, memory::ByteCount};

use crate::{process::Process, syscalls::structures::UserspaceAddress, kprint};

pub fn write(proc: &mut Process, _file_descriptor: usize, buffer: UserspaceAddress, length: ByteCount) -> Result<usize, SyscallError> {
    let ptr = proc.kernel_pointer(buffer)? as *mut u8;

    unsafe {
        for i in 0..length.raw_bytes() {
            kprint!("{}", ptr.add(i).read() as char);
        }
    }
    
    Ok(length.raw_bytes())
}