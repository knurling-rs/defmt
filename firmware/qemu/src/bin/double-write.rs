#![no_std]
#![no_main]

use cortex_m_rt::entry;

use defmt::Format;
use defmt_semihosting as _; // global logger

use cortex_m_semihosting::debug;

struct MyStruct;

impl Format for MyStruct {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "one");
        defmt::write!(fmt, "two");
    }
}

#[entry]
fn main() -> ! {
    defmt::info!("{:?}", MyStruct);

    loop {
        debug::exit(debug::EXIT_FAILURE);
    }
}

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        debug::exit(debug::EXIT_SUCCESS)
    }
}
