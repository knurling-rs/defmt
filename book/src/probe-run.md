# `probe-run`

(Temporary location for the docs of the `probe-run` tool.)

`probe-run` is a custom Cargo runner based on [probe-rs] that:

[probe-rs]: https://probe.rs/

- flashes Rust firmware onto a microcontroller
- collects `defmt` logs while the firmware runs, and
- prints a backtrace when the device halts
