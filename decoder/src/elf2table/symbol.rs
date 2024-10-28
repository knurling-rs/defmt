use std::borrow::Cow;

use anyhow::bail;
use serde::Deserialize;

use crate::Tag;

#[derive(Deserialize, PartialEq, Eq, Hash)]
pub(super) struct Symbol {
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

    /// Crate name obtained via CARGO_CRATE_NAME (added since a Cargo package can contain many crates).
    crate_name: Option<String>,
}

pub(super) enum SymbolTag {
    /// `defmt_*` tag that we can interpret.
    Defmt(Tag),

    /// Non-`defmt_*` tag for custom tooling.
    Custom(()),
}

impl Symbol {
    pub fn demangle(raw: &str) -> anyhow::Result<Self> {
        let mut raw = Cow::from(raw);
        if let Some(s) = raw.strip_prefix("__defmt_hex_") {
            raw = Cow::from(hex_decode(s)?);
        }
        serde_json::from_str(&raw)
            .map_err(|j| anyhow::anyhow!("failed to demangle defmt symbol `{}`: {}", raw, j))
    }

    pub fn tag(&self) -> SymbolTag {
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
            _ => SymbolTag::Custom(()),
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

    pub fn crate_name(&self) -> Option<&str> {
        self.crate_name.as_deref()
    }
}

fn hex_decode_digit(c: u8) -> anyhow::Result<u8> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 0xa),
        _ => bail!("invalid hex char '{c}'"),
    }
}

fn hex_decode(s: &str) -> anyhow::Result<String> {
    let s = s.as_bytes();
    if s.len() % 2 != 0 {
        bail!("invalid hex: length must be even")
    }
    let mut res = vec![0u8; s.len() / 2];
    for i in 0..(s.len() / 2) {
        res[i] = (hex_decode_digit(s[i * 2])? << 4) | hex_decode_digit(s[i * 2 + 1])?;
    }
    Ok(String::from_utf8(res)?)
}
