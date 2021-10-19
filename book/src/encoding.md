# Encoding

> 💡 Most users won't need to change the encoding so this section is mainly informative.

`defmt` data can be encoded using one of these 2 formats:

- `rzcobs` - [Reverse Zero-compressing COBS encoding][rzcobs] (rzCOBS). This is the default encoding.
- `raw` - raw data, that is no encoding.

[rzcobs]: https://github.com/Dirbaio/rzcobs

In comparison to not using any encoding, `rzcobs` compresses the data (uses less transport bandwidth),
and adds some degree of error detection thanks to its use of frames.

The encoding is selected via a Cargo feature on the `defmt` crate.
These Cargo features are named `encoding-{encoder_name}`, e.g. `encoding-rzcobs` and `encoding-raw`.

``` toml
[package]
name = "my-application"

[dependencies.defmt]
version = "0.3.0"
features = ["encoding-rzcobs"] # <- encoding
```

> ⚠️ Note that libraries (dependencies) MUST not select the encoder so that applications (top-level crates) can.

If no `enocding-*` feature is enabled then the default encoding is used.

The encoding is included in the output binary artifact as metadata so [printers](printers.html) will detect it and use the appropriate decoder automatically.
When the `rzcobs` encoding is used the printers will skip malformed frames (decoding errors) and continue decoding the rest of the `defmt` data.
In contrast, printers handling the `raw` encoding will exit on any decoding error.
