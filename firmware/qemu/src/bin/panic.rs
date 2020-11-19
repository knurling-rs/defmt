#![no_std]
#![no_main]

use cortex_m_rt::entry;

use defmt_semihosting as _; // global logger

#[entry]
fn main() -> ! {
    defmt::panic!("The answer is {:?}", 42);
}

// like `panic-semihosting` but doesn't print to stdout (that would corrupt the defmt stream)
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m_semihosting::debug::exit(debug::EXIT_SUCCESS)
    }
}
