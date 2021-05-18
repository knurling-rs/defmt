#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::debug;

use defmt::dbg;
use defmt_semihosting as _; // global logger

#[entry]
fn main() -> ! {
    // return value
    let x: i32 = 42;
    foo(dbg!(x + 1));

    // dbg! in log statement
    defmt::info!("the answer is {}", dbg!(x - 1));

    loop {
        debug::exit(debug::EXIT_SUCCESS)
    }
}

fn foo(_: i32) {}

// like `panic-semihosting` but doesn't print to stdout (that would corrupt the defmt stream)
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        debug::exit(debug::EXIT_FAILURE)
    }
}
