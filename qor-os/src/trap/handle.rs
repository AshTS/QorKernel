use crate::process::{processes, Process};

use super::{
    external::handle_external_interrupt,
    structures::{AsynchronousTrap, SynchronousTrap, TrapCause, TrapInfo},
};

#[allow(clippy::module_name_repetitions)]
pub fn handle_trap(info: &TrapInfo) -> usize {
    #[allow(clippy::match_single_binding)]
    match info.cause {
        TrapCause::AsynchronousTrap(AsynchronousTrap::MachineTimer) => {
            debug!("Machine timer interrupt");
            crate::drivers::CLINT_DRIVER.handle_interrupt(info.hart.into());

            let mut lock = crate::process::processes().spin_lock();
            if let Some(entry) = lock.first_entry() {
                let switching_data = entry.get().get_switching_data();
                drop(lock);

                Process::switch(switching_data);
            }

        }
        TrapCause::AsynchronousTrap(AsynchronousTrap::MachineExternal) => {
            handle_external_interrupt(info);
        }
        TrapCause::Synchronous(SynchronousTrap::Breakpoint) => {
            debug!("Breakpoint at 0x{:x}", info.trap_pc);
        }
        TrapCause::Synchronous(SynchronousTrap::EnvironmentCallFromUMode) => {
            warn!("Syscall!");
            warn!("PID: {:?}", qor_riscv::trap::get_pid());

            let pid = qor_riscv::trap::get_pid();
            let mut lock = processes().spin_lock();

            #[allow(clippy::option_if_let_else)]
            if let Some(proc) = lock.get_mut(&pid) {
                crate::syscalls::handler::raw_handle_syscall(proc);
            }
            else {
                error!("Got syscall from non-existant process {:?}", pid);
            }
        }
        _ => {
            panic!("Unhandled trap: {:x?}", info);
        }
    }

    if matches!(info.cause, TrapCause::Synchronous(_)) {
        info.trap_pc + 4
    } else {
        info.trap_pc
    }
}
