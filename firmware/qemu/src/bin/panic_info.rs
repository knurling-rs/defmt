#![no_std]
#![no_main]

use cortex_m as _;

use core::panic;
use semihosting::process::ExitCode;

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
    ExitCode::SUCCESS.exit_process()
}
