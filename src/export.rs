pub use defmt::info;

pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt()
    }
}
