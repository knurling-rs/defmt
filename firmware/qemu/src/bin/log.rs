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
    binfmt::info!("Hello {1:u16} {0:u8}", 42u8, 256u16);
    binfmt::info!("🍕 {:[u8]}", [3,14]);

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
