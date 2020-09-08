use cortex_m_rt as _;
pub use defmt::info;

pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt()
    }
}
