#![no_std]
#![no_main]

use core::u8::MAX;
use defmt_semihosting as _; // global logger

#[defmt_test::tests]
mod tests {
    use super::MAX;
    #[test]
    fn assert_true() {
        defmt::assert!(true)
    }

    #[test]
    fn assert_imported_max() {
        defmt::assert_eq!(255, MAX)
    }

    #[test]
    fn assert_eq() {
        defmt::assert_eq!(24, 42, "TODO: write actual tests")
    }
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
