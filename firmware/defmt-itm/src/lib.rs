//! [`defmt`](https://github.com/knurling-rs/defmt) global logger over ITM.
//!
//! To use this crate, call the `enable` function before using the defmt logging macros
//!
//! ``` no_run
//! // src/main.rs or src/bin/my-app.rs
//!
//! let p = cortex_m::Peripherals::take().unwrap();
//! defmt_itm::enable(p.ITM);
//!
//! defmt::info!("Hello");
//! ```

#![no_std]

use core::{
    ptr::NonNull,
    sync::atomic::{AtomicBool, Ordering},
};

use cortex_m::{interrupt, itm, peripheral::ITM, register};

static ENABLED: AtomicBool = AtomicBool::new(false);

/// Enables defmt logging over the ITM stimulus port 0.
///
/// This needs to be called by the application before defmt logging is used, otherwise the logs will be disposed.
pub fn enable(itm: ITM) {
    // enable stimulus port 0
    unsafe { itm.ter[0].write(1) }
    drop(itm);
    ENABLED.store(true, Ordering::Relaxed);
}

#[defmt::global_logger]
struct Logger;

impl defmt::Write for Logger {
    fn write(&mut self, bytes: &[u8]) {
        // NOTE(unsafe) this function will be invoked *after* `enable` has run so this crate now has
        // ownership over the ITM thus it's OK to instantiate the ITM register block here
        unsafe { itm::write_all(&mut (*ITM::ptr()).stim[0], bytes) }
    }
}

static TAKEN: AtomicBool = AtomicBool::new(false);
static INTERRUPTS_ACTIVE: AtomicBool = AtomicBool::new(false);

unsafe impl defmt::Logger for Logger {
    fn acquire() -> Option<NonNull<dyn defmt::Write>> {
        if !ENABLED.load(Ordering::Relaxed) {
            return None;
        }

        let primask = register::primask::read();
        interrupt::disable();
        if !TAKEN.load(Ordering::Relaxed) {
            // no need for CAS because interrupts are disabled
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
        TAKEN.store(false, Ordering::Relaxed);
        if INTERRUPTS_ACTIVE.load(Ordering::Relaxed) {
            // re-enable interrupts
            interrupt::enable()
        }
    }
}
