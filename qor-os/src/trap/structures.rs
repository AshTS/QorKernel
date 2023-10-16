#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapCause {
    Synchronous(SynchronousTrap),
    AsynchronousTrap(AsynchronousTrap),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SynchronousTrap {
    InstructionAddressMisaligned,
    InstructionAccessFault,
    IllegalInstruction,
    Breakpoint,
    LoadAddressMisaligned,
    LoadAccessFault,
    StoreAddressMisaligned,
    StoreAccessFault,
    EnvironmentCallFromUMode,
    EnvironmentCallFromSMode,
    EnvironmentCallFromMMode,
    InstructionPageFault,
    LoadPageFault,
    StorePageFault,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsynchronousTrap {
    UserSoftware,
    SupervisorSoftware,
    MachineSoftware,
    UserTimer,
    SupervisorTimer,
    MachineTimer,
    UserExternal,
    SupervisorExternal,
    MachineExternal,
}

#[derive(Debug, Clone)]
pub struct TrapInfo {
    pub cause: TrapCause,
    pub trap_value: usize,
    pub trap_pc: usize,
    pub hart: usize,
    pub status: usize,
    pub frame: &'static qor_riscv::trap::frame::TrapFrame,
}

impl TrapCause {
    pub const fn from_cause(cause: usize) -> Option<Self> {
        match cause {
            0 => Some(Self::Synchronous(
                SynchronousTrap::InstructionAddressMisaligned,
            )),
            1 => Some(Self::Synchronous(SynchronousTrap::InstructionAccessFault)),
            2 => Some(Self::Synchronous(SynchronousTrap::IllegalInstruction)),
            3 => Some(Self::Synchronous(SynchronousTrap::Breakpoint)),
            4 => Some(Self::Synchronous(SynchronousTrap::LoadAddressMisaligned)),
            5 => Some(Self::Synchronous(SynchronousTrap::LoadAccessFault)),
            6 => Some(Self::Synchronous(SynchronousTrap::StoreAddressMisaligned)),
            7 => Some(Self::Synchronous(SynchronousTrap::StoreAccessFault)),
            8 => Some(Self::Synchronous(SynchronousTrap::EnvironmentCallFromUMode)),
            9 => Some(Self::Synchronous(SynchronousTrap::EnvironmentCallFromSMode)),
            11 => Some(Self::Synchronous(SynchronousTrap::EnvironmentCallFromMMode)),
            12 => Some(Self::Synchronous(SynchronousTrap::InstructionPageFault)),
            13 => Some(Self::Synchronous(SynchronousTrap::LoadPageFault)),
            15 => Some(Self::Synchronous(SynchronousTrap::StorePageFault)),
            0x8000_0000_0000_0000 => Some(Self::AsynchronousTrap(AsynchronousTrap::UserSoftware)),
            0x8000_0000_0000_0001 => {
                Some(Self::AsynchronousTrap(AsynchronousTrap::SupervisorSoftware))
            }
            0x8000_0000_0000_0003 => {
                Some(Self::AsynchronousTrap(AsynchronousTrap::MachineSoftware))
            }
            0x8000_0000_0000_0004 => Some(Self::AsynchronousTrap(AsynchronousTrap::UserTimer)),
            0x8000_0000_0000_0005 => {
                Some(Self::AsynchronousTrap(AsynchronousTrap::SupervisorTimer))
            }
            0x8000_0000_0000_0007 => Some(Self::AsynchronousTrap(AsynchronousTrap::MachineTimer)),
            0x8000_0000_0000_0008 => Some(Self::AsynchronousTrap(AsynchronousTrap::UserExternal)),
            0x8000_0000_0000_0009 => {
                Some(Self::AsynchronousTrap(AsynchronousTrap::SupervisorExternal))
            }
            0x8000_0000_0000_000b => {
                Some(Self::AsynchronousTrap(AsynchronousTrap::MachineExternal))
            }
            _ => None,
        }
    }
}

impl TrapInfo {
    pub fn from_raw(
        epc: usize,
        trap_value: usize,
        cause: usize,
        hart: usize,
        status: usize,
        frame: &'static qor_riscv::trap::frame::TrapFrame,
    ) -> Self {
        Self {
            cause: TrapCause::from_cause(cause).expect("Invalid trap cause"),
            trap_value,
            trap_pc: epc,
            hart,
            status,
            frame,
        }
    }
}
