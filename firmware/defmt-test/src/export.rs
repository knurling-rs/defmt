pub use defmt::info;

use crate::TestOutcome;

/// Terminates the application and reports successful exit to the debugger.
///
/// Uses ARM semihosting to signal program termination. First attempts the extended
/// exit operation for better exit code support, then falls back to the standard
/// operation if unsupported by the debugger.
pub fn exit() -> ! {
    semihosting::process::exit(0);
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
