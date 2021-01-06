#![no_std]
#![no_main]

use cortex_m_rt::entry;

use defmt_semihosting as _; // global logger

#[entry]
fn main() -> ! {
    defmt::assert!({ 1 + 1 } == { 2 });

    let dev = false;
    let release = false;
    defmt::debug_assert!(dev);
    defmt::assert!(release);
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
