#![no_std]
#![no_main]

use defmt_semihosting as _; // global logger

#[defmt_test::tests]
mod tests {
    use core::u8::MAX;
    use defmt::{assert, assert_eq};

    #[init]
    fn init() {}

    #[test]
    fn assert_true() -> () {
        assert!(true);
    }

    #[test]
    fn assert_imported_max() {
        assert_eq!(255, MAX);
    }

    #[cfg(not(never))]
    #[test]
    fn result() -> Result<(), ()> {
        Ok(())
    }

    #[cfg(never)]
    #[test]
    fn doesnt_compile() {
        because::this::doesnt::exist();
    }

    #[test]
    #[should_error]
    fn should_error() -> Result<(), ()> {
        Err(())
    }

    #[test]
    #[should_error]
    fn fail() -> Result<&'static str, ()> {
        Ok("this should have returned `Err`")
    }
}

// like `panic-semihosting` but doesn't print to stdout (that would corrupt the defmt stream)
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    use cortex_m_semihosting::debug;

    loop {
        debug::exit(debug::EXIT_FAILURE)
    }
}
