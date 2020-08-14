#![no_std]
#![no_main]

use defmt::Format;
use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m_rt::entry;
use cortex_m_semihosting::debug;

use defmt_semihosting as _; // global logger
use panic_halt as _; // panicking behavior

#[entry]
fn main() -> ! {
    defmt::info!("Hello!");
    defmt::info!("World!");
    defmt::info!("The answer is {:u8}", 42);
    defmt::info!("Hello {0:u8} {0:u8}!", 42);
    defmt::info!("Hello {1:u16} {0:u8} {2:bool}", 42u8, 256u16, false);
    defmt::info!("🍕 slice {:[u8]}", [3, 14]);
    defmt::info!("🍕 array {:[u8; 3]}", [3, 14, 1]);
    defmt::info!("float like a butterfly {:f32}", 5.67f32);
    defmt::info!("Hello {:u8}", 42u16 as u8);

    defmt::info!(
        "isize: 0 = {:isize}, -1 = {:isize}, MAX = {:isize}, MIN = {:isize}",
        0,
        -1,
        isize::max_value(),
        isize::min_value()
    );
    defmt::info!(
        "isize: 0 = {:?}, -1 = {:?}, MAX = {:?}, MIN = {:?}",
        0,
        -1,
        isize::max_value(),
        isize::min_value()
    );
    defmt::info!("usize: 0 = {:usize}, MAX = {:usize}", 0, usize::max_value());
    defmt::info!("bitfields {0:0..3} {0:5..7}", 0b0110_0011_1101_0110u16);

    defmt::trace!("log trace");
    defmt::debug!("log debug");
    defmt::info!("log info");
    defmt::warn!("log warn");
    defmt::error!("log error");

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

    defmt::info!("{:?}", S { x: 1, y: 256 });
    defmt::info!("{:?}", X { y: Y { z: 42 } });

    let interned = defmt::intern!("interned string");
    defmt::info!("&str = {:str}", "string slice");
    defmt::info!("&Str = {:istr}", interned);

    #[derive(Format)]
    struct Arr {
        arr1: [u8; 1],
        arr0: [u8; 0],
        arr32: [u8; 32],
    }

    defmt::info!(
        "{:?}",
        Arr {
            arr1: [0x1f],
            arr0: [],
            arr32: [0x55; 32]
        }
    );

    let slice: &[u16] = &[256, 257, 258];
    defmt::info!("{:[?]}", slice);

    let ss: &[S] = &[S { x: 128, y: 256 }, S { x: 129, y: 257 }];
    defmt::info!("{:[?]}", ss);

    let xs: &[X] = &[X { y: Y { z: 128 } }, X { y: Y { z: 129 } }];
    defmt::info!("{:[?]}", xs);

    let slices: &[&[u16]] = &[&[256, 257, 258], &[259, 260]];
    defmt::info!("{:[?]}", slices);

    #[derive(Format)]
    enum E {
        A,
        B,
    }

    defmt::info!("e1={:?}", E::A);
    defmt::info!("e2={:?}", E::B);

    defmt::info!("e3={:?}", Some(42u8));
    defmt::info!("e4={:?}", None::<u8>);

    defmt::info!("e5={:?}", Ok::<u8, u16>(42u8));
    defmt::info!("e6={:?}", Err::<u8, u16>(256u16));

    defmt::info!("e7={:?}", Some(X { y: Y { z: 42 } }));

    loop {
        debug::exit(debug::EXIT_SUCCESS)
    }
}

#[defmt::timestamp]
fn timestamp() -> u64 {
    // monotonic counter
    static COUNT: AtomicU32 = AtomicU32::new(0);
    COUNT.fetch_add(1, Ordering::Relaxed) as u64
}
