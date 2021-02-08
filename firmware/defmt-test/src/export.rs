use cortex_m_rt as _;
pub use defmt::info;

use crate::TestOutcome;

pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt()
    }
}

pub fn check_outcome<T: TestOutcome>(outcome: T, should_error: bool) {
    if outcome.is_success() == should_error {
        let note = if should_error {
            defmt::intern!("`#[should_error]` ")
        } else {
            defmt::intern!("")
        };
        defmt::panic!("{}test failed with outcome: {}", note, outcome);
    }
}
