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

use cortex_m::{
    asm, itm,
    peripheral::{itm::Stim, ITM},
};

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
    ENABLED.store(true, Ordering::Relaxed);
}

#[defmt::global_logger]
struct Logger;

static TAKEN: AtomicBool = AtomicBool::new(false);
static mut CS_RESTORE: critical_section::RestoreState = critical_section::RestoreState::invalid();
static mut ENCODER: defmt::Encoder = defmt::Encoder::new();

unsafe impl defmt::Logger for Logger {
    fn acquire() {
        // safety: Must be paired with corresponding call to release(), see below
        let restore = unsafe { critical_section::acquire() };

        // safety: accessing the atomic without CAS is OK because we have acquired a critical section.
        if TAKEN.load(Ordering::Relaxed) {
            panic!("defmt logger taken reentrantly")
        }

        // safety: accessing the atomic without CAS is OK because we have acquired a critical section.
        TAKEN.store(true, Ordering::Relaxed);

        // safety: accessing the `static mut` is OK because we have acquired a critical section.
        unsafe { CS_RESTORE = restore };

        // safety: accessing the `static mut` is OK because we have acquired a critical section.
        unsafe {
            let encoder: &mut defmt::Encoder = &mut *core::ptr::addr_of_mut!(ENCODER);
            encoder.start_frame(do_write)
        }
    }

    unsafe fn flush() {
        // wait for the queue to be able to accept more data
        while !stim_0().is_fifo_ready() {}

        // delay "a bit" to drain the queue
        // This is a heuristic and might be too short in reality. Please open an issue if it is!
        asm::delay(100);
    }

    unsafe fn release() {
        // safety: accessing the `static mut` is OK because we have acquired a critical section.
        unsafe {
            let encoder: &mut defmt::Encoder = &mut *core::ptr::addr_of_mut!(ENCODER);
            encoder.end_frame(do_write);
        }

        // safety: accessing the atomic without CAS is OK because we have acquired a critical section.
        TAKEN.store(false, Ordering::Relaxed);

        // safety: accessing the `static mut` is OK because we have acquired a critical section.
        let restore = unsafe { CS_RESTORE };

        // safety: Must be paired with corresponding call to acquire(), see above
        unsafe {
            critical_section::release(restore);
        }
    }

    unsafe fn write(bytes: &[u8]) {
        // safety: accessing the `static mut` is OK because we have acquired a critical section.
        unsafe {
            let encoder: &mut defmt::Encoder = &mut *core::ptr::addr_of_mut!(ENCODER);
            encoder.write(bytes, do_write);
        }
    }
}

fn do_write(bytes: &[u8]) {
    // NOTE(unsafe) this function will be invoked *after* `enable` has run so this crate now has
    // ownership over the ITM thus it's OK to instantiate the ITM register block here
    unsafe { itm::write_all(stim_0(), bytes) }
}

/// Get access to stimulus port 0
///
/// # Safety
/// Can only be invoked *after* `enable` has run
unsafe fn stim_0<'a>() -> &'a mut Stim {
    &mut (*ITM::PTR).stim[0]
}
