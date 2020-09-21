use std::collections::hash_map::DefaultHasher;
use std::env;
use std::hash::Hash;
use std::hash::Hasher;

use proc_macro::Span;
use serde::Serialize;

#[derive(Serialize)]
pub struct Symbol<'a> {
    /// Name of the Cargo package in which the symbol is being instantiated. Used for avoiding
    /// symbol name collisions.
    package: String,

    /// Unique identifier that disambiguates otherwise equivalent invocations in the same crate.
    disambiguator: u64,

    /// Symbol categorization. Known values:
    /// * `defmt_prim` for primitive formatting strings that are placed at the start of the `.defmt`
    ///   section.
    /// * `defmt_fmt`, `defmt_str` for interned format strings and string literals.
    /// * `defmt_trace`, `defmt_debug`, `defmt_info`, `defmt_warn`, `defmt_error` for logging
    ///   messages used at the different log levels.
    /// * Anything starting with `defmt_` is reserved for use by defmt, other prefixes are free for
    ///   use by third-party apps (but they all should use a prefix!).
    tag: String,

    /// Symbol data for use by the host tooling. Interpretation depends on `tag`.
    data: &'a str,
}

impl<'a> Symbol<'a> {
    pub fn new(tag: &'a str, data: &'a str) -> Self {
        Self {
            // `CARGO_PKG_NAME` is set to the invoking package's name.
            package: env::var("CARGO_PKG_NAME").unwrap(),
            disambiguator: {
                // We want a deterministic, but unique-per-macro-invocation identifier. For that we
                // hash the call site `Span`'s debug representation, which contains a counter that
                // should disambiguate macro invocations within a crate.
                let s = format!("{:?}", Span::call_site());
                let mut hasher = DefaultHasher::new();
                s.hash(&mut hasher);
                hasher.finish()
            },
            tag: format!("defmt_{}", tag),
            data,
        }
    }

    pub fn mangle(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
