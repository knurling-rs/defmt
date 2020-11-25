#![no_std]
#![no_main]

use cortex_m_rt::entry;

use defmt_semihosting as _; // global logger

#[entry]
fn main() -> ! {
    defmt::assert_eq!({1 + 1}, { 2 });

    let x = 42;
    defmt::debug_assert_eq!(x - 1, x + 1, "dev");
    defmt::assert_eq!(x - 1, x + 1, "release");
    defmt::unreachable!();
}

// like `panic-semihosting` but doesn't print to stdout (that would corrupt the defmt stream)
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    use cortex_m_semihosting::debug;

    loop {
        debug::exit(debug::EXIT_SUCCESS)
    }
}
