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

            if let Some(entry) = crate::process::processes().spin_lock().first_entry() {
                entry.get().switch_to();
            }

        }
        TrapCause::AsynchronousTrap(AsynchronousTrap::MachineExternal) => {
            handle_external_interrupt(info);
        }
        TrapCause::Synchronous(SynchronousTrap::Breakpoint) => {
            debug!("Breakpoint at 0x{:x}", info.trap_pc);
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
