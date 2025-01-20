# Defmt Versions

Any given version of the `defmt` crate will implement one specific *Defmt
Version* - also known as the *wire format version*. Because a compilation can
only use one version of the `defmt` crate, each compilation will use a
consistent *Defmt Version*.

The *Defmt Version* used in any given ELF file is expressed using symbol (listed below).

The `defmt-decoder` crate supports multiple *Defmt Version* values, and so can
work with newer and older firmware.

This is a list of *Defmt Version* values and what they mean.

## Defmt Version 4

- Supported by defmt-decoder versions: 0.3.6 onwards
- Supported by defmt-print versions: 0.3.6 onwards
- Supported by defmt versions: 0.3.4 onwards
- Symbol name: `_defmt_version_ = 4`
- Interned strings are JSON, with fields:
  - `package`: the name of the package (as defined in `Cargo.toml`) that emitted the log
  - `tag`: one of `defmt_prim`, `defmt_fmt`, `defmt_str`, `defmt_trace`, `defmt_debug`, `defmt_info`, `defmt_warn`, or `defmt_error`
  - `data`: the format string
  - `disambiguator`: a unique random number
  - `crate_name`: the crate the emitted the log (might not be the package name if this is a binary within a package)
- Supported encodings:
  - RZCOBS: with symbol `_defmt_encoding_ = rzcobs`
  - RAW: with symbol `_defmt_encoding_ = raw`
- Withdrawal notice: This version will be supported in new releases of the `defmt-decoder` and `defmt-print` crates for at least the next 24 months, on a rolling basis. Notice will be given here when that 24 month period begins.

## Defmt Version 3

- First stable wire format (before that we used crate versions / git hashes)
- Supported by defmt versions: 0.3.0 to 0.3.3
- Interned strings are JSON, with fields:
  - `package`: the name of the package (as defined in `Cargo.toml`) that emitted the log
  - `tag`: one of `defmt_prim`, `defmt_fmt`, `defmt_str`, `defmt_trace`, `defmt_debug`, `defmt_info`, `defmt_warn`, or `defmt_error`
  - `data`: the format string
  - `disambiguator`: a unique random number
- Symbol name: `_defmt_version_ = 3`
- Withdrawal notice: This version will be supported in new releases of the `defmt-decoder` and `defmt-print` crates for at least the next 12 months, on a rolling basis. Notice will be given here when that 12 month period begins.

## PUA Defmt Versions

We have set-aside a range of Defmt Versions for private use. We guarantee that no official version of the defmt tools will use these versions, so you can unambiguously use them to add customised features to your private forks of defmt.

The reserved versions are `_defmt_version_ = 1000` through to `_defmt_version_ = 1999`.
