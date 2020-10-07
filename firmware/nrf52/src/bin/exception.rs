#![no_std]
#![no_main]

use core::sync::atomic::{AtomicUsize, Ordering};

use cortex_m::{asm, peripheral::SCB};
use cortex_m_rt::{entry, exception};
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
    defmt::info!("main");
    SCB::set_pendsv();
    defmt::info!("after PendSV");

    loop {
        asm::bkpt()
    }
}

#[exception]
fn PendSV() {
    defmt::info!("PendSV");
    asm::bkpt()
}
