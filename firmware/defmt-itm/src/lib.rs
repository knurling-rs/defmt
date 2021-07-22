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

#![doc(html_logo_url = "https://knurling.ferrous-systems.com/knurling_logo_light_text.svg")]
#![no_std]

use core::sync::atomic::{AtomicBool, Ordering};

use cortex_m::{interrupt, itm, peripheral::ITM, register};

#[cfg(armv6m)]
compile_error!(
    "`defmt-itm` cannot be used on Cortex-M0(+) chips, because it requires an ITM peripheral"
);

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

static TAKEN: AtomicBool = AtomicBool::new(false);
static INTERRUPTS_ACTIVE: AtomicBool = AtomicBool::new(false);
static mut ENCODER: defmt::Encoder = defmt::Encoder::new();

unsafe impl defmt::Logger for Logger {
    fn acquire() {
        if !ENABLED.load(Ordering::Relaxed) {
            panic!("defmt ITM logger is not enabled")
        }

        let primask = register::primask::read();
        interrupt::disable();

        if TAKEN.load(Ordering::Relaxed) {
            panic!("defmt logger taken reentrantly")
        }

        // no need for CAS because interrupts are disabled
        TAKEN.store(true, Ordering::Relaxed);

        INTERRUPTS_ACTIVE.store(primask.is_active(), Ordering::Relaxed);

        // safety: accessing the `static mut` is OK because we have disabled interrupts.
        unsafe { ENCODER.start_frame(do_write) }
    }

    unsafe fn flush() {
        todo!()
    }

    unsafe fn release() {
        // safety: accessing the `static mut` is OK because we have disabled interrupts.
        ENCODER.end_frame(do_write);

        TAKEN.store(false, Ordering::Relaxed);
        if INTERRUPTS_ACTIVE.load(Ordering::Relaxed) {
            // re-enable interrupts
            interrupt::enable()
        }
    }

    unsafe fn write(bytes: &[u8]) {
        // safety: accessing the `static mut` is OK because we have disabled interrupts.
        ENCODER.write(bytes, do_write);
    }
}

fn do_write(bytes: &[u8]) {
    // NOTE(unsafe) this function will be invoked *after* `enable` has run so this crate now has
    // ownership over the ITM thus it's OK to instantiate the ITM register block here
    unsafe { itm::write_all(&mut (*ITM::ptr()).stim[0], bytes) }
}
