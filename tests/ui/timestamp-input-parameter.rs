#![no_main]
#[defmt::timestamp]
fn foo(x: u64) -> u64 { x + 1 }
