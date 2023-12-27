//! Panic handler for `probe-rs`.
//!
//! When this panic handler is used, panics will make `probe-rs` print a backtrace (by triggering a semihosting::process::abort).
//! Probe-rs will then exit with a non-zero status code, indicating failure.
//! This building block can be used to run on-device tests.
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

#[cfg(all(not(cortex_m), not(target_arch = "riscv32")))]
compile_error!("`panic-probe` only supports Cortex-M targets or riscv32");

// Functionality `cfg`d out on platforms with OS/libstd.
#[cfg(target_os = "none")]
mod imp {
    use core::panic::PanicInfo;
    use core::sync::atomic::{AtomicBool, Ordering};

    #[cfg(feature = "print-rtt")]
    use crate::print_rtt::print;

    #[cfg(feature = "print-defmt")]
    use crate::print_defmt::print;

    #[cfg(feature = "print-log")]
    use crate::print_log::print;

    #[cfg(not(any(feature = "print-rtt", feature = "print-defmt", feature = "print-log")))]
    fn print(_: &core::panic::PanicInfo) {}

    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        critical_section::with(|_| {
            // Guard against infinite recursion, just in case.
            static PANICKED: AtomicBool = AtomicBool::new(false);
            if !PANICKED.load(Ordering::Relaxed) {
                PANICKED.store(true, Ordering::Relaxed);

                print(info);
            }

            crate::abort() // this call will never return, therefore we stay in the critical section forever.
        })
    }
}

/// Triggers a semihosting::process::abort
///
/// This function may be used to as `defmt::panic_handler` to avoid double prints.
///
/// # Examples
///
/// ```
/// #[defmt::panic_handler]
/// fn panic() -> ! {
///     panic_probe::abort();
/// }
/// ```
#[cfg(target_os = "none")]
pub fn abort() -> ! {
    semihosting::process::abort();
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

#[cfg(feature = "print-log")]
mod print_log {
    use core::panic::PanicInfo;

    pub fn print(info: &PanicInfo) {
        log::error!("{}", info);
    }
}
