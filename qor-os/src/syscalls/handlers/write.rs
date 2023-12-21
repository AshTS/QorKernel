use qor_core::{structures::syscall_error::SyscallError, memory::ByteCount, tasks::Task};

use crate::{process::Process, syscalls::structures::UserspaceAddress};

pub fn write(proc: &mut Process, file_descriptor: usize, buffer: UserspaceAddress, length: ByteCount) -> Result<usize, SyscallError> {
    let ptr = proc.kernel_pointer(buffer)? as *mut u8;

    let file_descriptor = proc.file_descriptor(file_descriptor)?.clone();

    unsafe {
        qor_core::tasks::execute_task(Task::ignore_result(file_descriptor.write(core::slice::from_raw_parts(ptr, length.raw_bytes()))));
    }
    
    Ok(length.raw_bytes())
}