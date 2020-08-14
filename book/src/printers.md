# Printers

*Printers* are *host* programs that receive log data, format it and display it.
The following printers are currently available:

- `qemu-run`, parses data sent by QEMU over semihosting (ARM Cortex-M only)
- [`probe-run`], parses data sent over RTT (ARM Cortex-M only). NOTE: make sure you install the tool from git (not crates.io) and enable the "defmt" Cargo feature.

[`probe-run`]: https://github.com/knurling-rs/probe-run
