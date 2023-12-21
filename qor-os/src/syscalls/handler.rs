use qor_core::memory::ByteCount;

use crate::{process::Process, syscalls::{handlers, structures::UserspaceAddress}};

use super::structures::SyscallNumber;

pub fn raw_handle_syscall(proc: &mut Process) {

    let syscall_number = proc.registers()[17];
    
    #[allow(clippy::option_if_let_else)]
    if let Some(syscall) = SyscallNumber::from_number(syscall_number) {
        let result = match syscall {
            SyscallNumber::Write => handlers::write::write(proc, 
                proc.registers()[10].try_into().unwrap(),
            UserspaceAddress(proc.registers()[11].try_into().unwrap()), 
            ByteCount::new(proc.registers()[12].try_into().unwrap())),
            _ => todo!()
        };

        debug!("{:?}", result);

        match result {
            Ok(value) => { proc.registers_mut()[10] = value as u64 },
            Err(value) => {
                let e: isize = value.into();
                proc.registers_mut()[10] = u64::from_ne_bytes(i64::to_ne_bytes(e as i64));
            }
        }
    }
    else {
        panic!("Unknown syscall number {}", syscall_number);
    }
}