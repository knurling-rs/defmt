#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::debug;

use defmt_semihosting as _; // global logger

#[derive(defmt::Format)]
enum Error {
    Bar,
}

#[entry]
fn main() -> ! {
    let x: Result<u32, Error> = Ok(42);
    defmt::info!("The answer is {:?}", defmt::unwrap!(x));
    let x: Result<u32, Error> = Err(Error::Bar);
    defmt::info!("The answer is {:?}", defmt::unwrap!(x));

    loop {
        debug::exit(debug::EXIT_SUCCESS)
    }
}

// like `panic-semihosting` but doesn't print to stdout (that would corrupt the defmt stream)
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        debug::exit(debug::EXIT_SUCCESS)
    }
}
