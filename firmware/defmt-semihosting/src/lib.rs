//! `defmt` global logger over semihosting
//!
//! NOTE this is meant to only be used with QEMU
//!
//! WARNING using `cortex_m_semihosting`'s `hprintln!` macro or `HStdout` API will corrupt `defmt`
//! log frames so don't use those APIs.
//!
//! # Critical section implementation
//!
//! This crate uses [`critical-section`](https://github.com/rust-embedded/critical-section) to ensure only one thread
//! is writing to the buffer at a time. You must import a crate that provides a `critical-section` implementation
//! suitable for the current target. See the `critical-section` README for details.
//!
//! For example, for single-core privileged-mode Cortex-M targets, you can add the following to your Cargo.toml.
//!
//! ```toml
//! [dependencies]
//! cortex-m = { version = "0.7.6", features = ["critical-section-single-core"]}
//! ```

#![no_std]

use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, Ordering},
};

use cortex_m_semihosting::hio;

struct ContextInner {
    restore_state: critical_section::RestoreState,
    encoder: defmt::Encoder,
}

struct Context {
    taken: AtomicBool,
    inner: UnsafeCell<ContextInner>,
}

// safety: assumes contents are accessed under an isolating critical section.
unsafe impl Sync for Context {}

#[defmt::global_logger]
struct Logger;

static CONTEXT: Context = Context {
    taken: AtomicBool::new(false),
    inner: UnsafeCell::new(ContextInner {
        restore_state: critical_section::RestoreState::invalid(),
        encoder: defmt::Encoder::new(),
    }),
};

unsafe impl defmt::Logger for Logger {
    fn acquire() {
        // safety: Must be paired with corresponding call to release(), see below
        let restore = unsafe { critical_section::acquire() };

        if CONTEXT.taken.load(Ordering::Relaxed) {
            panic!("defmt logger taken reentrantly")
        }

        // no need for CAS because interrupts are disabled
        CONTEXT.taken.store(true, Ordering::Relaxed);

        // safety: this assumes the critical section we're in isolates execution
        unsafe {
            let inner = &mut *CONTEXT.inner.get();
            inner.restore_state = restore;
            inner.encoder.start_frame(do_write);
        }
    }

    unsafe fn flush() {
        // Do nothing.
        //
        // semihosting is fundamentally blocking, and does not have I/O buffers the target can control.
        // After write returns, the host has the data, so there's nothing left to flush.
    }

    unsafe fn release() {
        // safety: this assumes the critical section we're in isolates execution
        let restore = unsafe {
            let inner = &mut *CONTEXT.inner.get();
            inner.encoder.end_frame(do_write);
            inner.restore_state
        };
        CONTEXT.taken.store(false, Ordering::Relaxed);

        // safety: Must be paired with corresponding call to acquire(), see above
        unsafe { critical_section::release(restore) };
    }

    unsafe fn write(bytes: &[u8]) {
        // safety: this assumes the critical section we're in isolates execution
        unsafe { (*CONTEXT.inner.get()).encoder.write(bytes, do_write) };
    }
}

fn do_write(bytes: &[u8]) {
    // using QEMU; it shouldn't mind us opening several handles (I hope)
    if let Ok(mut hstdout) = hio::hstdout() {
        hstdout.write_all(bytes).ok();
    }
}
