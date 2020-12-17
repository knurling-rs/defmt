#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};

use cortex_m_rt::entry;
use cortex_m_semihosting::debug;

use defmt::{intern, write, Format, Formatter};
use defmt_semihosting as _; // global logger

#[entry]
fn main() -> ! {
    let x = 42;
    defmt::info!("no hint {=u8}", x);
    defmt::info!("hex     {=u8:x}", x);
    defmt::info!("HEX     {=u8:X}", x);
    defmt::info!("binary  {=u8:b}", x);
    defmt::info!("ASCII   {=u8:a}", x);
    defmt::info!("Debug   {=u8:?}", x);

    defmt::info!("----");

    let x = 42;
    defmt::info!("no-hint {=i8}", x);
    defmt::info!("hex     {=i8:x}", x);
    defmt::info!("HEX     {=i8:X}", x);
    defmt::info!("binary  {=i8:b}", x);
    defmt::info!("ASCII   {=i8:a}", x);
    defmt::info!("Debug   {=i8:?}", x);

    defmt::info!("----");

    // no type information
    // the hint should propagate downwards into the `Format` implementation
    // the `Format` implementation of `i8` uses `{=i8}` as its format string
    defmt::info!("no hint {}", x);
    defmt::info!("hex     {:x}", x);
    defmt::info!("HEX     {:X}", x);
    defmt::info!("binary  {:b}", x);
    defmt::info!("ASCII   {:a}", x);
    defmt::info!("Debug   {:?}", x);

    defmt::info!("----");

    // hints bind tightly
    {
        struct S1;

        impl Format for S1 {
            fn format(&self, f: Formatter) {
                write!(f, "{:x}", S2)
            }
        }

        struct S2;

        impl Format for S2 {
            fn format(&self, f: Formatter) {
                // innermost hint has precedence
                // outer ':x' will be ignored
                write!(f, "{:b}", 42)
            }
        }

        defmt::info!("S1 > S2 {}", S1);
    }

    defmt::info!("----");

    let x = b"He\x7Fllo";
    defmt::info!("no hint {=[u8]}", *x);
    defmt::info!("hex     {=[u8]:x}", *x);
    defmt::info!("HEX     {=[u8]:X}", *x);
    defmt::info!("binary  {=[u8]:b}", *x);
    defmt::info!("ASCII   {=[u8]:a}", *x);
    defmt::info!("Debug   {=[u8]:?}", *x);

    defmt::info!("----");

    {
        let mut bytes = [0; 256];
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = i as u8;
        }

        defmt::info!("{=[u8]:a}", bytes);
        defmt::info!("{=[u8;256]:a}", bytes);
    }

    defmt::info!("----");

    let s = "Hello";
    let is = intern!("world");
    defmt::info!("no hint {=str}", s);
    defmt::info!("Debug   {=str:?}", s);

    defmt::info!("no hint {=istr}", is);
    defmt::info!("Debug   {=istr:?}", is);

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

// like `panic-semihosting` but doesn't print to stdout (that would corrupt the defmt stream)
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        debug::exit(debug::EXIT_FAILURE)
    }
}
