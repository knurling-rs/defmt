# Printers

*Printers* are *host* programs that receive log data, format it and display it.
The following printers are currently available:

- [`probe-run`], parses data sent over RTT (ARM Cortex-M only).
  > 💡 If you are using the git version of defmt, make sure you also install the tool from git and not crates.io.
- [`defmt-print`], a generic command-line tool that decodes defmt data passed into its standard input.
- [`qemu-run`], parses data sent by QEMU over semihosting (ARM Cortex-M only).
  > 💡 Used for internal testing and won't be published to crates.io

[`probe-run`]: https://github.com/knurling-rs/probe-run
[`defmt-print`]: https://github.com/knurling-rs/defmt/tree/main/print
[`qemu-run`]: https://github.com/knurling-rs/defmt/tree/main/qemu-run
