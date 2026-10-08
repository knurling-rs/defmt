# Printers

*Printers* are *host* programs that receive log data, format it and display it.
The following printers are currently available:

- [`probe-rs`], comprehensive embedded toolkit, decoding and displaying defmt data sent over RTT.

  > 💡 This is the recommended way to get started with embedded Rust and defmt.

- [`qemu-run`], parses data sent by QEMU over semihosting.

  Available on crates.io; install with `cargo install qemu-run`.
  Requires `qemu-system-arm` to be installed separately.

- [`defmt-print`], a generic command-line tool that decodes defmt data passed into its standard input.
- [`probe-run`], parses data sent over RTT (ARM Cortex-M only).

  **Deprecated:** Use [`probe-rs`] instead.

[`probe-rs`]: https://probe.rs/
[`probe-run`]: https://github.com/knurling-rs/probe-run
[`defmt-print`]: https://github.com/knurling-rs/defmt/tree/main/print
[`qemu-run`]: https://crates.io/crates/qemu-run
[`--json`]: ./json-output.md
[addressed here]: https://github.com/knurling-rs/defmt/issues/664

> 💡 If you are using an experimental version of defmt, consider using the same version when installing the printer tool.
