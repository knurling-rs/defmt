use crate::Tag;
use serde::Deserialize;

#[derive(Deserialize, PartialEq, Eq, Hash)]
pub struct Symbol {
    /// Name of the Cargo package in which the symbol is being instantiated. Used for avoiding
    /// symbol name collisions.
    package: String,

    /// Unique identifier that disambiguates otherwise equivalent invocations in the same crate.
    disambiguator: String,

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
    data: String,

    /// https://github.com/knurling-rs/defmt/issues/532
    /// TODO BM: Write proper description
    crate_name: String,
}

pub enum SymbolTag<'a> {
    /// `defmt_*` tag that we can interpret.
    Defmt(Tag),

    /// Non-`defmt_*` tag for custom tooling.
    Custom(&'a str),
}

impl Symbol {
    pub fn demangle(raw: &str) -> anyhow::Result<Self> {
        serde_json::from_str(raw)
            .map_err(|j| anyhow::anyhow!("failed to demangle defmt symbol `{}`: {}", raw, j))
    }

    pub fn tag(&self) -> SymbolTag<'_> {
        match &*self.tag {
            "defmt_prim" => SymbolTag::Defmt(Tag::Prim),
            "defmt_derived" => SymbolTag::Defmt(Tag::Derived),
            "defmt_bitflags" => SymbolTag::Defmt(Tag::Bitflags),
            "defmt_write" => SymbolTag::Defmt(Tag::Write),
            "defmt_timestamp" => SymbolTag::Defmt(Tag::Timestamp),
            "defmt_bitflags_value" => SymbolTag::Defmt(Tag::BitflagsValue),
            "defmt_str" => SymbolTag::Defmt(Tag::Str),
            "defmt_println" => SymbolTag::Defmt(Tag::Println),
            "defmt_trace" => SymbolTag::Defmt(Tag::Trace),
            "defmt_debug" => SymbolTag::Defmt(Tag::Debug),
            "defmt_info" => SymbolTag::Defmt(Tag::Info),
            "defmt_warn" => SymbolTag::Defmt(Tag::Warn),
            "defmt_error" => SymbolTag::Defmt(Tag::Error),
            _ => SymbolTag::Custom(&self.tag),
        }
    }

    pub fn data(&self) -> &str {
        &self.data
    }

    pub fn package(&self) -> &str {
        &self.package
    }

    pub fn disambiguator(&self) -> &str {
        &self.disambiguator
    }

    pub fn crate_name(&self) -> &str {
        &self.crate_name
    }
}
