#![no_std]
#![no_main]

use binfmt_rtt as _; // <- global logger
use cortex_m_rt::entry;
use nrf52840_hal as _; // <- memory layout
use panic_probe as _; // <- panicking behavior

#[binfmt::timestamp]
fn timestamp() -> u64 {
    0
}

#[entry]
fn main() -> ! {
    binfmt::info!("main");

    panic!()
}
