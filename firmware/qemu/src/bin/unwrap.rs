#![no_std]
#![no_main]

use cortex_m as _;
use cortex_m_rt::entry;
use semihosting::process::ExitCode;

use defmt_semihosting as _; // global logger

#[derive(defmt::Format)]
enum Error {
    Bar,
}

#[entry]
fn main() -> ! {
    let x: Result<u32, Error> = Ok(42);
    defmt::info!("The answer is {=?}", defmt::unwrap!(x));
    let x: Result<u32, Error> = Err(Error::Bar);
    defmt::info!("The answer is {=?}", defmt::unwrap!(x));

    ExitCode::SUCCESS.exit_process()
}

// like `panic-semihosting` but doesn't print to stdout (that would corrupt the defmt stream)
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    ExitCode::SUCCESS.exit_process()
}
