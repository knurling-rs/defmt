#![no_std]
#![no_main]

use defmt_rtt as _; // <- global logger
use cortex_m::asm;
use cortex_m_rt::entry;
use nrf52840_hal as _; // <- memory layout
use panic_probe as _; // <- panicking behavior

#[defmt::timestamp]
fn timestamp() -> u64 {
    0
}

#[entry]
fn main() -> ! {
    defmt::info!("main");
    assert!(false);
    loop {
        asm::bkpt()
    }
}
