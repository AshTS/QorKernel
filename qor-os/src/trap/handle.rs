use qor_core::{drivers::timer::HardwareTimerDriver, structures::time::Hertz};

use super::structures::{AsynchronousTrap, SynchronousTrap, TrapCause, TrapInfo};

#[allow(clippy::module_name_repetitions)]
pub fn handle_trap(info: &TrapInfo) -> usize {
    #[allow(clippy::match_single_binding)]
    match info.cause {
        TrapCause::AsynchronousTrap(AsynchronousTrap::MachineTimer) => {
            debug!("Machine timer interrupt");
            crate::drivers::CLINT_DRIVER
                .set_time_rate(info.hart.into(), Hertz(3))
                .expect("Unable to set the CLINT Timer rate");
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
