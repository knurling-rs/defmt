//! `defmt` global logger over semihosting
//!
//! NOTE this is meant to only be used with QEMU
//!
//! WARNING using `cortex_m_semihosting`'s `hprintln!` macro or `HStdout` API will corrupt `defmt`
//! log frames so don't use those APIs.

#![no_std]
// nightly warns about static_mut_refs, but 1.76 (our MSRV) does not know about
// that lint yet, so we ignore it it but also ignore if it is unknown
#![allow(unknown_lints, static_mut_refs)]

use core::sync::atomic::{AtomicBool, Ordering};

use cortex_m::{interrupt, register};
use cortex_m_semihosting::hio;

#[defmt::global_logger]
struct Logger;

static TAKEN: AtomicBool = AtomicBool::new(false);
static mut CS_RESTORE: critical_section::RestoreState = critical_section::RestoreState::invalid();
static mut ENCODER: defmt::Encoder = defmt::Encoder::new();

unsafe impl defmt::Logger for Logger {
    fn acquire() {
        // safety: Must be paired with corresponding call to release(), see below
        let restore = unsafe { critical_section::acquire() };

        if TAKEN.load(Ordering::Relaxed) {
            panic!("defmt logger taken reentrantly")
        }

        // no need for CAS because interrupts are disabled
        TAKEN.store(true, Ordering::Relaxed);

        // safety: accessing the `static mut` is OK because we have acquired a critical section.
        unsafe { CS_RESTORE = restore };

        // safety: accessing the `static mut` is OK because we have disabled interrupts.
        unsafe { ENCODER.start_frame(do_write) }
    }

    unsafe fn flush() {
        // Do nothing.
        //
        // semihosting is fundamentally blocking, and does not have I/O buffers the target can control.
        // After write returns, the host has the data, so there's nothing left to flush.
    }

    unsafe fn release() {
        // safety: accessing the `static mut` is OK because we have disabled interrupts.
        ENCODER.end_frame(do_write);

        TAKEN.store(false, Ordering::Relaxed);

        // safety: accessing the `static mut` is OK because we have acquired a critical section.
        let restore = unsafe { CS_RESTORE };

        // safety: Must be paired with corresponding call to acquire(), see above
        unsafe { critical_section::release(restore) };
    }

    unsafe fn write(bytes: &[u8]) {
        // safety: accessing the `static mut` is OK because we have disabled interrupts.
        ENCODER.write(bytes, do_write);
    }
}

fn do_write(bytes: &[u8]) {
    // using QEMU; it shouldn't mind us opening several handles (I hope)
    if let Ok(mut hstdout) = hio::hstdout() {
        hstdout.write_all(bytes).ok();
    }
}
