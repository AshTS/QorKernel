use super::structures::TrapInfo;

#[allow(clippy::module_name_repetitions)]
pub fn handle_trap(info: &TrapInfo) {
    #[allow(clippy::match_single_binding)]
    match info.cause {
        _ => {
            panic!("Unhandled trap: {:?}", info);
        }
    }
}
