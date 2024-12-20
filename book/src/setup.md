# Setup

To use `defmt` in a **library** you only need to add `defmt` as a dependency.

```console
$ cargo add defmt
```

To use `defmt` in an **application** you need the additional steps documented below.

## For applications

> 💡 The prefferred way to create a *new* `defmt` application is to use our [app-template]. Tag along if you want to add `defmt` to an *existing* application.

[app-template]: https://github.com/knurling-rs/app-template

### Linker script

The application must be linked using a custom linking process that includes the `defmt.x` linker script.
Custom linking is usual for embedded applications and configured in the `.cargo/config` file.

To pass `defmt.x` to the linker add the `-C link-arg=-Tdefmt.x` flag to the rustflags section of `.cargo/config.toml`.

``` toml
# .cargo/config.toml
[target.thumbv7m-none-eabi]
rustflags = [
  # --- KEEP existing `link-arg` flags ---
  "-C", "link-arg=-Tlink.x",
  "-C", "link-arg=--nmagic",

  # --- ADD following new flag ---
  "-C", "link-arg=-Tdefmt.x",
]
```

### `#[global_logger]`

The application must link to or define a `global_logger`.
The `global_logger` specifies how logs are sent from the device running the app to the host where the logs are displayed.
The application must link to exactly one `global_logger`.
The `global_logger` can appear anywhere in the dependency graph and usually it will be its own crate.
The following `global_logger`s are provided as part of the project:

- [`defmt-rtt`], logs over RTT. Note that this crate can *not* be used together with `rtt-target`.
- [`defmt-itm`], logs over ITM (Instrumentation Trace Macrocell) stimulus port 0.
- [`defmt-semihosting`], logs over semihosting. Meant only for testing `defmt` on a virtual Cortex-M device (QEMU).

[`defmt-rtt`]: https://docs.rs/defmt-rtt/
[`defmt-itm`]: https://docs.rs/defmt-itm/
[`defmt-semihosting`]: https://github.com/knurling-rs/defmt/tree/6cfd947384debb18a4df761cbe454f8d86cf3441/firmware/defmt-semihosting

Information about how to write a `global_logger` can be found in the [`#[global_logger]` section](./global-logger.md).

### Enabling logging

By default, only ERROR level messages are logged.
To learn how to enable other logging levels and filters logs per modules read the [Filtering section](./filtering.md).

### Memory use

When in a tight memory situation and logging over RTT, the buffer size (default: 1024 bytes) can be configured with the `DEFMT_RTT_BUFFER_SIZE` environment variable. Use a power of 2 for best performance.

### Use on Windows/macOS/Linux

You can use `defmt` without string-interning by enabling the `no-interning` feature. This will mean that format strings are included as literal strings and not hidden in the symbol name.

You can optionally send the output to the `log` crate by enabling the `use-log-crate` feature.
