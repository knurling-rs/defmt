#![no_std]
#![no_main]

use cortex_m as _;
use cortex_m_rt::entry;
use semihosting::process::ExitCode;

use defmt_rtt as _; // global logger

#[entry]
fn main() -> ! {
    defmt::info!("drop-on-contention");

    ExitCode::SUCCESS.exit_process()
}

// like `panic-semihosting` but doesn't print to stdout (that would corrupt the defmt stream)
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    ExitCode::FAILURE.exit_process()
}
