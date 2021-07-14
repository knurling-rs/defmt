#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::debug;
use defmt::{bitflags, Debug2Format};

use defmt_semihosting as _; // global logger

bitflags! {
    struct Flags: u8 {
        #[cfg(not(never))]
        const FLAG_0 = 0b00;
        const FLAG_1 = 0b01;
        const FLAG_2 = 0b10;
        const FLAG_7 = 1 << 7;

        #[cfg(never)]
        const CFGD_OUT = 1;
    }
}

bitflags! {
    struct LargeFlags: u128 {
        const MSB = 1 << 127;
        const ALL = !0;
        const NON_LITERAL = compute_flag_value(0x346934553632462);
    }
}

const fn compute_flag_value(x: u128) -> u128 {
    x ^ 0xdeadbeef
}

#[entry]
fn main() -> ! {
    defmt::info!("Flags::empty(): {}", Flags::empty());
    defmt::info!(
        "Flags::empty(): {} (fmt::Debug)",
        Debug2Format(&Flags::empty())
    );
    defmt::info!("Flags::all(): {}", Flags::all());
    defmt::info!("Flags::all(): {} (fmt::Debug)", Debug2Format(&Flags::all()));
    defmt::info!("Flags::FLAG_1: {}", Flags::FLAG_1);
    defmt::info!(
        "Flags::FLAG_1: {} (fmt::Debug)",
        Debug2Format(&Flags::FLAG_1)
    );
    defmt::info!("Flags::FLAG_7: {}", Flags::FLAG_7);
    defmt::info!(
        "Flags::FLAG_7: {} (fmt::Debug)",
        Debug2Format(&Flags::FLAG_7)
    );

    defmt::info!("LargeFlags::ALL: {}", LargeFlags::ALL);
    defmt::info!(
        "LargeFlags::ALL: {} (fmt::Debug)",
        Debug2Format(&LargeFlags::ALL)
    );
    defmt::info!("LargeFlags::empty(): {}", LargeFlags::empty());
    defmt::info!(
        "LargeFlags::empty(): {} (fmt::Debug)",
        Debug2Format(&LargeFlags::empty())
    );

    loop {
        debug::exit(debug::EXIT_SUCCESS)
    }
}

// like `panic-semihosting` but doesn't print to stdout (that would corrupt the defmt stream)
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        debug::exit(debug::EXIT_FAILURE)
    }
}
