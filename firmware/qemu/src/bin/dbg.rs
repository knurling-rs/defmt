#![no_std]
#![no_main]

use cortex_m as _;
use cortex_m_rt::entry;
use semihosting::process::ExitCode;

use defmt::dbg;
use defmt_semihosting as _; // global logger

#[entry]
fn main() -> ! {
    // return value
    let x: i32 = 42;
    #[allow(clippy::double_parens)]
    foo(dbg!(x + 1));

    // dbg! in log statement
    defmt::info!("the answer is {}", dbg!(x - 1));

    // dbg! with multiple arguments
    let _: (i32, i32) = dbg!(x - 2, x + 2);

    // dbg! with zero arguments
    #[allow(clippy::let_unit_value)]
    let () = dbg!();

    // dbg! with trailing comma
    #[allow(clippy::double_parens)]
    foo(dbg!(x,));

    ExitCode::SUCCESS.exit_process()
}

fn foo(_: i32) {}

// like `panic-semihosting` but doesn't print to stdout (that would corrupt the defmt stream)
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    ExitCode::FAILURE.exit_process()
}
