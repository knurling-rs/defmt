#![no_std]
#![no_main]

use cortex_m as _;
use cortex_m_rt::entry;
use semihosting::process::ExitCode;

use defmt_semihosting as _; // global logger

#[entry]
fn main() -> ! {
    let owner = 42u8;
    let count = 0x1234u16;

    // inline capture from the surrounding scope
    defmt::info!("{owner=u8}");
    defmt::info!("{owner}");
    defmt::info!("{owner:?}");
    defmt::info!("{owner=u8:#x} then {count=u16:#x}");

    // repeated use of one capture serializes the argument only once
    defmt::info!("{owner} repeated {owner}");

    // explicitly supplied named arguments
    defmt::info!("explicit {x=u8}", x = 1);

    // mixed positional and named
    defmt::info!("mixed pos={=u8} named={owner=u8} first={0=u8}", 7);

    defmt::println!("println {owner=u8} {count=u16:#x}");

    ExitCode::SUCCESS.exit_process()
}

// like `panic-semihosting` but doesn't print to stdout (that would corrupt the defmt stream)
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    ExitCode::FAILURE.exit_process()
}
