use qor_core::structures::id::PID;

pub mod frame;

#[must_use]
pub fn get_pid() -> PID {
    #[allow(clippy::cast_possible_truncation)]
    ((riscv::register::satp::read().asid() & 0xffff) as u16).into()
}