#![no_std]
#![no_main]

use cortex_m::asm;
use cortex_m_rt::entry;
use defmt_rtt as _; // <- global logger
use nrf52840_hal as _; // <- memory layout
use panic_probe as _; // <- panicking behavior

#[entry]
fn main() -> ! {
    defmt::info!("main");
    assert!(false);
    loop {
        asm::bkpt()
    }
}
