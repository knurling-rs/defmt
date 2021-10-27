#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};

use defmt_semihosting as _; // global logger

static MAY_PANIC: AtomicBool = AtomicBool::new(false);

#[defmt_test::tests]
mod tests {
    use core::{sync::atomic::Ordering, u8::MAX};
    use defmt::{assert, assert_eq};

    struct InitStruct {
        test: u8,
    }

    #[repr(C)]
    #[derive(Debug)]
    struct SomeData {
        elem1: u8,
        elem2: f32,
    }

    #[init]
    fn init() -> InitStruct {
        InitStruct { test: 8 }
    }

    #[test]
    fn change_init_struct(init_struct: &mut InitStruct) {
        assert_eq!(init_struct.test, 8);
        init_struct.test = 42;
    }

    #[test]
    fn test_for_changed_init_struct(init_struct: &mut InitStruct) {
        assert_eq!(init_struct.test, 42);
    }

    #[test]
    fn assert_true() -> () {
        assert!(true);
    }

    const CUSTOM_MAX: u8 = 255;

    #[test]
    fn assert_imported_max() {
        assert_eq!(CUSTOM_MAX, MAX);
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
    #[ignore]
    #[allow(dead_code)]
    fn ignored() -> Result<(), ()> {
        Err(())
    }

    #[test]
    #[should_error]
    fn fail() -> Result<&'static str, ()> {
        // this test is expected to fail (= panic)
        super::MAY_PANIC.store(true, Ordering::Relaxed);

        Ok("this should have returned `Err`")
    }
}

// like `panic-semihosting` but doesn't print to stdout (that would corrupt the defmt stream)
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    use cortex_m_semihosting::debug;

    loop {
        let exit_code = if MAY_PANIC.load(Ordering::Relaxed) {
            debug::EXIT_SUCCESS
        } else {
            debug::EXIT_FAILURE
        };
        debug::exit(exit_code)
    }
}
