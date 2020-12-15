#![no_main]
#![no_std]

#[defmt::timestamp]
#[no_mangle]
fn foo() -> u64 {
    0
}
