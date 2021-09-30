#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::debug;
use defmt::{write, Format, Formatter};

use defmt_semihosting as _; // global logger

#[entry]
fn main() -> ! {
    defmt::info!("test {=bool}", true);

    defmt::println!("Hello {}{}", "World", '!');

    loop {
        debug::exit(debug::EXIT_SUCCESS)
    }
}

struct Timestamp {
    h: u8,
    m: u8,
    s: u8,
}

impl Format for Timestamp {
    fn format(&self, fmt: Formatter<'_>) {
        write!(fmt, "It is {}:{}:{} {=bool}", self.h, self.m, self.s, true);
    }
}

defmt::timestamp!(
    "{} {=bool}",
    Timestamp {
        h: 10,
        m: 20,
        s: 30
    },
    false,
);

// like `panic-semihosting` but doesn't print to stdout (that would corrupt the defmt stream)
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        debug::exit(debug::EXIT_FAILURE)
    }
}
