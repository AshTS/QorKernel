use super::structures::{TrapInfo, TrapCause, SynchronousTrap};

#[allow(clippy::module_name_repetitions)]
pub fn handle_trap(info: &TrapInfo) -> usize {
    #[allow(clippy::match_single_binding)]
    match info.cause {
        TrapCause::Synchronous(SynchronousTrap::Breakpoint) => {
            debug!("Breakpoint at 0x{:x}", info.trap_pc);
        },
        _ => {
            panic!("Unhandled trap: {:?}", info);
        }
    }

    if matches!(info.cause, TrapCause::Synchronous(_)) {
        info.trap_pc + 4
    } else {
        info.trap_pc
    }
}
