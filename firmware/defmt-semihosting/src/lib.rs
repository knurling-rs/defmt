//! `defmt` global logger over semihosting
//!
//! WARNING using `semihosting`'s `println!` macro or `Stdout` API will corrupt
//! `defmt` log frames so don't use those APIs.
//!
//! # Critical section implementation
//!
//! This crate uses
//! [`critical-section`](https://github.com/rust-embedded/critical-section) to
//! ensure only one thread is writing to the buffer at a time. You must import a
//! crate that provides a `critical-section` implementation suitable for the
//! current target. See the `critical-section` README for details.
//!
//! For example, for single-core privileged-mode Cortex-M targets, you can add
//! the following to your Cargo.toml.
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

use semihosting::io::{Stdout, Write as _};

#[defmt::global_logger]
struct Logger;

static SEMIHOSTING_ENCODER: SemihostingEncoder = SemihostingEncoder::new();

struct SemihostingEncoder {
    /// A boolean lock
    ///
    /// Is `true` when `acquire` has been called and we have exclusive access to
    /// the rest of this structure.
    taken: AtomicBool,
    /// We need to remember this to exit a critical section
    cs_restore: UnsafeCell<critical_section::RestoreState>,
    /// A defmt::Encoder for encoding frames
    encoder: UnsafeCell<defmt::Encoder>,
    /// A semihosting handle for outputting encoded data
    handle: UnsafeCell<Option<Stdout>>,
}

impl SemihostingEncoder {
    /// Create a new semihosting-based defmt-encoder
    const fn new() -> SemihostingEncoder {
        SemihostingEncoder {
            taken: AtomicBool::new(false),
            cs_restore: UnsafeCell::new(critical_section::RestoreState::invalid()),
            encoder: UnsafeCell::new(defmt::Encoder::new()),
            handle: UnsafeCell::new(None),
        }
    }

    /// Acquire the defmt encoder.
    fn acquire(&self) {
        // safety: Must be paired with corresponding call to release(), see below
        let restore = unsafe { critical_section::acquire() };

        // NB: You can re-enter critical sections but we need to make sure
        // no-one does that.
        if self.taken.load(Ordering::Relaxed) {
            panic!("defmt logger taken reentrantly")
        }

        // no need for CAS because we are in a critical section
        self.taken.store(true, Ordering::Relaxed);

        // safety: accessing the cell is OK because we have acquired a critical
        // section.
        unsafe {
            self.cs_restore.get().write(restore);
            let encoder: &mut defmt::Encoder = &mut *self.encoder.get();
            let handle: &mut Option<Stdout> = &mut *self.handle.get();
            if handle.is_none() {
                *handle = semihosting::io::stdout().ok();
            }
            encoder.start_frame(|b| {
                if let Some(h) = handle {
                    _ = h.write_all(b);
                }
            });
        }
    }

    /// Release the defmt encoder.
    unsafe fn release(&self) {
        if !self.taken.load(Ordering::Relaxed) {
            panic!("defmt release out of context")
        }

        // safety: accessing the cell is OK because we have acquired a critical
        // section.
        unsafe {
            let encoder: &mut defmt::Encoder = &mut *self.encoder.get();
            let handle: &mut Option<Stdout> = &mut *self.handle.get();
            encoder.end_frame(|b| {
                if let Some(h) = handle {
                    _ = h.write_all(b);
                }
            });
            let restore = self.cs_restore.get().read();
            self.taken.store(false, Ordering::Relaxed);
            // paired with exactly one acquire call
            critical_section::release(restore);
        }
    }

    /// Write bytes to the defmt encoder.
    unsafe fn write(&self, bytes: &[u8]) {
        if !self.taken.load(Ordering::Relaxed) {
            panic!("defmt write out of context")
        }

        // safety: accessing the cell is OK because we have acquired a critical
        // section.
        unsafe {
            let encoder: &mut defmt::Encoder = &mut *self.encoder.get();
            let handle: &mut Option<Stdout> = &mut *self.handle.get();
            encoder.write(bytes, |b| {
                if let Some(h) = handle {
                    _ = h.write_all(b);
                }
            });
        }
    }
}

unsafe impl Sync for SemihostingEncoder {}

unsafe impl defmt::Logger for Logger {
    fn acquire() {
        SEMIHOSTING_ENCODER.acquire();
    }

    unsafe fn flush() {
        // Do nothing.
        //
        // semihosting is fundamentally blocking, and does not have I/O buffers the target can control.
        // After write returns, the host has the data, so there's nothing left to flush.
    }

    unsafe fn release() {
        unsafe {
            SEMIHOSTING_ENCODER.release();
        }
    }

    unsafe fn write(bytes: &[u8]) {
        unsafe {
            SEMIHOSTING_ENCODER.write(bytes);
        }
    }
}
