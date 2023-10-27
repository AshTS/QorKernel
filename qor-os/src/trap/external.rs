use qor_core::{drivers::plic::PLICDriverInterface, interfaces::bytes::GenericByteReadInterface};

use crate::{
    drivers::{PLIC_DRIVER, UART_DRIVER, UART_INTERRUPT},
    kprint,
};

use super::structures::TrapInfo;

/// Function which is executed when an external interrupt is triggered
pub fn handle_external_interrupt(info: &TrapInfo) {
    #[allow(clippy::option_if_let_else)]
    if let Some(interrupt_id) = PLIC_DRIVER
        .poll_interrupt(info.hart.into())
        .expect("Unable to poll PLIC")
    {
        match interrupt_id {
            UART_INTERRUPT => {
                if let Some(byte) = UART_DRIVER
                    .read_byte()
                    .expect("Unable to read byte from UART")
                {
                    kprint!("{}", byte as char);
                }
            }
            _ => {
                panic!("Unhandled interrupt: {:?}", interrupt_id);
            }
        }

        PLIC_DRIVER
            .complete_interrupt(info.hart.into(), interrupt_id)
            .expect("Unable to complete interrupt");
    } else {
        panic!("No interrupt found, this is unexpected");
    }
}
