#![no_std]
#![no_main]

use binfmt::Format;
use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m_rt::entry;
use cortex_m_semihosting::debug;

use binfmt_semihosting as _; // global logger
use panic_halt as _; // panicking behavior

#[entry]
fn main() -> ! {
    binfmt::info!("Hello!");
    binfmt::info!("World!");
    binfmt::info!("The answer is {:u8}", 42);
    binfmt::info!("Hello {0:u8} {0:u8}!", 42);
    binfmt::info!("Hello {1:u16} {0:u8} {:bool}", 42u8, 256u16, false);
    binfmt::info!("🍕 slice {:[u8]}", [3, 14]);
    binfmt::info!("🍕 array {:[u8; 3]}", [3, 14, 1]);
    binfmt::info!("float like a butterfly {:f32}", 5.67f32);

    binfmt::info!(
        "isize: 0 = {:isize}, -1 = {:isize}, MAX = {:isize}, MIN = {:isize}",
        0,
        -1,
        isize::max_value(),
        isize::min_value()
    );
    binfmt::info!(
        "isize: 0 = {:?}, -1 = {:?}, MAX = {:?}, MIN = {:?}",
        0,
        -1,
        isize::max_value(),
        isize::min_value()
    );
    binfmt::info!("usize: 0 = {:usize}, MAX = {:usize}", 0, usize::max_value());

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

    let interned = binfmt::intern!("interned string");
    binfmt::info!("&str = {:str}", "string slice");
    binfmt::info!("&Str = {:istr}", interned);

    #[derive(Format)]
    struct Arr {
        arr1: [u8; 1],
        arr0: [u8; 0],
        arr32: [u8; 32],
    }

    binfmt::info!(
        "{:?}",
        Arr {
            arr1: [0x1f],
            arr0: [],
            arr32: [0x55; 32]
        }
    );

    loop {
        debug::exit(debug::EXIT_SUCCESS)
    }
}

#[binfmt::timestamp]
fn timestamp() -> u64 {
    // monotonic counter
    static COUNT: AtomicU32 = AtomicU32::new(0);
    COUNT.fetch_add(1, Ordering::Relaxed) as u64
}
