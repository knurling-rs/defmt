//! Panic handler for `probe-run`.
//!
//! When this panic handler is used, panics will make `probe-run` print a backtrace and exit with a
//! non-zero status code, indicating failure. This building block can be used to run on-device
//! tests.
//!
//! # Panic Messages
//!
//! By default, `panic-probe` *ignores* the panic message. You can enable one of the following
//! features to print it instead:
//!
//! - `print-rtt`: Prints the panic message over plain RTT (via `rtt-target`). RTT must be
//!   initialized by the app.
//! - `print-defmt`: Prints the panic message via [defmt]'s transport (note that defmt will not be
//!   used to efficiently format the message).
//!
//! [defmt]: https://github.com/knurling-rs/defmt/

#![no_std]
#![cfg(target_os = "none")]
#![doc(html_logo_url = "https://knurling.ferrous-systems.com/knurling_logo_light_text.svg")]

#[cfg(not(cortex_m))]
compile_error!("`panic-probe` only supports Cortex-M targets (thumbvN-none-eabi[hf])");

// Functionality `cfg`d out on platforms with OS/libstd.
#[cfg(target_os = "none")]
mod imp {
    use core::panic::PanicInfo;
    use core::sync::atomic::{AtomicBool, Ordering};

    use cortex_m::asm;

    #[cfg(feature = "print-rtt")]
    use crate::print_rtt::print;

    #[cfg(feature = "print-defmt")]
    use crate::print_defmt::print;

    #[cfg(not(any(feature = "print-rtt", feature = "print-defmt")))]
    fn print(_: &core::panic::PanicInfo) {}

    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        static PANICKED: AtomicBool = AtomicBool::new(false);

        cortex_m::interrupt::disable();

        // Guard against infinite recursion, just in case.
        if !PANICKED.load(Ordering::Relaxed) {
            PANICKED.store(true, Ordering::Relaxed);

            print(info);
        }

        // Trigger a `HardFault` via `udf` instruction.

        // If `UsageFault` is enabled, we disable that first, since otherwise `udf` will cause that
        // exception instead of `HardFault`.
        #[cfg(not(any(armv6m, armv8m_base)))]
        {
            const SHCSR: *mut u32 = 0xE000ED24usize as _;
            const USGFAULTENA: usize = 18;

            unsafe {
                let mut shcsr = core::ptr::read_volatile(SHCSR);
                shcsr &= !(1 << USGFAULTENA);
                core::ptr::write_volatile(SHCSR, shcsr);
            }
        }

        asm::udf();
    }
}

#[cfg(feature = "print-rtt")]
mod print_rtt {
    use core::panic::PanicInfo;
    use rtt_target::rprintln;

    pub fn print(info: &PanicInfo) {
        rprintln!("{}", info);
    }
}

#[cfg(feature = "print-defmt")]
mod print_defmt {
    use core::panic::PanicInfo;

    pub fn print(info: &PanicInfo) {
        defmt::error!("{}", defmt::Display2Format(info));
    }
}
