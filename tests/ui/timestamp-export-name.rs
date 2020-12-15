#![no_main]
#![no_std]

#[defmt::timestamp]
#[export_name = "hello"]
fn foo() -> u64 {
    0
}
