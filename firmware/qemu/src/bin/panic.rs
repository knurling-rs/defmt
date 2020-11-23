#![no_std]
#![no_main]

use cortex_m_rt::entry;

use defmt_semihosting as _; // global logger

#[entry]
fn main() ->  ! {
    let answer = 42;
    let foo: u32 = match answer {
        1 => 123,
        2 => 456,
        _ => defmt::panic!("The answer is {:?}", answer),
    };
    defmt::panic!("should never get here {:?}", foo);
}

// like `panic-semihosting` but doesn't print to stdout (that would corrupt the defmt stream)
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    use cortex_m_semihosting::debug;

    loop {
        debug::exit(debug::EXIT_SUCCESS)
    }
}
