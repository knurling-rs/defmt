#![no_std]
#![no_main]

use core::sync::atomic::{AtomicUsize, Ordering};

use cortex_m::asm;
use cortex_m_rt::entry;
use defmt_rtt as _; // <- global logger
use nrf52840_hal as _; // <- memory layout
use panic_probe as _; // <- panicking behavior

#[defmt::timestamp]
fn timestamp() -> u64 {
    static N: AtomicUsize = AtomicUsize::new(0);
    N.fetch_add(1, Ordering::Relaxed) as u64
}

#[entry]
fn main() -> ! {
    defmt::info!("Hello, world!");

    loop {
        asm::bkpt()
    }
}
