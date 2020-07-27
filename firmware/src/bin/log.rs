#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;

// TODO add missing symbols

#[entry]
fn main() -> ! {
    binfmt::info!("Hello!");

    loop {
        // your code goes here
    }
}
