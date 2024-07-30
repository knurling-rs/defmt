#![no_std]
#![no_main]

use core::panic;
use cortex_m_semihosting::debug;

use defmt_semihosting as _; // global logger

#[cortex_m_rt::entry]
fn main() -> ! {
    // Note: this test is a bit brittle in that the line/column number of the following panic is
    // included in the test snapshot.  Hence, be mindful to update the snapshot if you want to
    // add any additional code to this file above the following line!
    panic!("aaah!")
}

#[panic_handler]
fn panic(panic_info: &panic::PanicInfo) -> ! {
    defmt::info!("PanicInfo: {=?}", panic_info);
    loop {
        debug::exit(debug::EXIT_SUCCESS)
    }
}
