//! `defmt` global logger over semihosting
//!
//! NOTE this is meant to only be used with QEMU
//!
//! WARNING using `cortex_m_semihosting`'s `hprintln!` macro or `HStdout` API will corrupt `defmt`
//! log frames so don't use those APIs.

#![no_std]

use core::{
    ptr::NonNull,
    sync::atomic::{AtomicBool, Ordering},
};

use cortex_m::{interrupt, register};
use cortex_m_semihosting::hio;

#[defmt::global_logger]
struct Logger;

impl defmt::Write for Logger {
    fn write(&mut self, bytes: &[u8]) {
        // using QEMU; it shouldn't mind us opening several handles (I hope)
        if let Ok(mut hstdout) = hio::hstdout() {
            hstdout.write_all(bytes).ok();
        }
    }
}

static TAKEN: AtomicBool = AtomicBool::new(false);
static INTERRUPTS_ACTIVE: AtomicBool = AtomicBool::new(false);

unsafe impl defmt::Logger for Logger {
    fn acquire() -> Option<NonNull<dyn defmt::Write>> {
        let primask = register::primask::read();
        interrupt::disable();

        if !TAKEN.load(Ordering::Relaxed) {
            // NOTE(no-CAS) interrupts are disabled
            TAKEN.store(true, Ordering::Relaxed);

            INTERRUPTS_ACTIVE.store(primask.is_active(), Ordering::Relaxed);

            Some(NonNull::from(&Logger as &dyn defmt::Write))
        } else {
            if primask.is_active() {
                // re-enable interrupts
                unsafe { interrupt::enable() }
            }
            None
        }
    }

    unsafe fn release(_: NonNull<dyn defmt::Write>) {
        // NOTE(no-CAS) interrupts still disabled
        TAKEN.store(false, Ordering::Relaxed);

        if INTERRUPTS_ACTIVE.load(Ordering::Relaxed) {
            // re-enable interrupts
            interrupt::enable()
        }
    }
}
