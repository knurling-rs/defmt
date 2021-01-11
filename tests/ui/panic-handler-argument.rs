#![no_main]
#![no_std]

use defmt_macros::info;
#[panic_handler]
fn foo(x: bool) -> ! { info!("{:?}", x); loop {} }
