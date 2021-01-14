use defmt_decoder::Tag;
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
}

pub enum SymbolTag<'a> {
    /// `defmt_*` tag that we can interpret.
    Defmt(Tag),

    /// Non-`defmt_*` tag for custom tooling.
    Custom(&'a str),
}

impl Symbol {
    pub fn demangle(raw: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(raw)
    }

    pub fn tag(&self) -> SymbolTag<'_> {
        match &*self.tag {
            "defmt_prim" | "defmt_fmt" => SymbolTag::Defmt(Tag::Fmt),
            "defmt_timestamp" => SymbolTag::Defmt(Tag::Timestamp),
            "defmt_str" => SymbolTag::Defmt(Tag::Str),
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
}
