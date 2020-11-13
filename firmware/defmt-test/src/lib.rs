//! A test harness for embedded devices
//!
//! This crate has a single API: the `#[tests]` macro. This macro is documented in the project
//! README which can be found at:
//!
//! - https://crates.io/crates/defmt (crates.io version)
//! - https://github.com/knurling-rs/defmt/tree/main/firmware/defmt-test (git version)

#![no_std]

pub use defmt_test_macros::tests;

pub mod export;
