#![no_std]
#![no_main]

use binfmt::Format;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hio};
use panic_halt as _;

#[entry]
fn main() -> ! {
    binfmt::info!("Hello!");
    binfmt::info!("World!");
    binfmt::info!("The answer is {:u8}", 42);

    binfmt::trace!("log trace");
    binfmt::debug!("log debug");
    binfmt::info!("log info");
    binfmt::warn!("log warn");
    binfmt::error!("log error");

    #[derive(Format)]
    struct S {
        x: u8,
        y: u16,
    }

    #[derive(Format)]
    struct X {
        y: Y,
    }

    #[derive(Format)]
    struct Y {
        z: u8,
    }

    binfmt::info!("{:?}", S { x: 1, y: 256 });
    binfmt::info!("{:?}", X { y: Y { z: 42 } });

    loop {
        debug::exit(debug::EXIT_SUCCESS)
    }
}

#[no_mangle]
fn _binfmt_timestamp() -> u64 {
    // monotonic counter
    static I: AtomicU32 = AtomicU32::new(0);
    I.fetch_add(1, Ordering::Relaxed) as u64
}

struct Logger;

impl binfmt::Write for Logger {
    fn write(&mut self, bytes: &[u8]) {
        // using QEMU; it shouldn't mind us opening several handles (I hope)
        if let Ok(mut hstdout) = hio::hstdout() {
            hstdout.write_all(bytes).ok();
        }
    }
}

static TAKEN: AtomicBool = AtomicBool::new(false);

#[no_mangle]
fn _binfmt_acquire() -> Option<binfmt::Formatter> {
    // NOTE: will lose data in presence of interrupts but not important ATM
    if TAKEN
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_ok()
    {
        Some(unsafe {
            binfmt::Formatter::from_raw(
                &Logger as &dyn binfmt::Write as *const dyn binfmt::Write as *mut dyn binfmt::Write,
            )
        })
    } else {
        None
    }
}

#[no_mangle]
fn _binfmt_release(_: binfmt::Formatter) {
    TAKEN.store(false, Ordering::Relaxed)
}
