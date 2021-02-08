//! A test harness for embedded devices.
//!
//! This crate has a single API: the `#[tests]` macro. This macro is documented in the project
//! README which can be found at:
//!
//! - <https://crates.io/crates/defmt-test> (crates.io version)
//! - <https://github.com/knurling-rs/defmt/tree/main/firmware/defmt-test> (git version)

#![no_std]

use defmt::Format;
pub use defmt_test_macros::tests;

/// Private implementation details used by the proc macro.
#[doc(hidden)]
pub mod export;

/// Types indicating whether a test succeeded or failed.
///
/// This is comparable to the `Termination` trait in libstd, except stable and tailored towards the
/// needs of defmt-test. It can be implemented by hand if desired, though normal usage shouldn't
/// require that.
pub trait TestOutcome: Format {
    fn is_success(&self) -> bool;
}

impl TestOutcome for () {
    fn is_success(&self) -> bool {
        true
    }
}

impl<T: Format, E: Format> TestOutcome for Result<T, E> {
    fn is_success(&self) -> bool {
        self.is_ok()
    }
}
