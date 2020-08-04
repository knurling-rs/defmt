#![no_std]

use core::panic::PanicInfo;

use cortex_m::asm;
use cortex_m_rt::{exception, ExceptionFrame};

#[panic_handler]
fn abort(_: &PanicInfo) -> ! {
    // trigger a `HardFault`
    asm::udf()
}

#[exception]
fn HardFault(_: &ExceptionFrame) -> ! {
    loop {
        // make `probe-run` print the backtrace and exit
        asm::bkpt()
    }
}
