# Application setup

> ⚠️ Remember to also do the base setup from the previous chapter!

## Linker script

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

## `#[global_logger]`

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

## Enabling logging

By default, only ERROR level messages are logged.
To learn how to enable other logging levels and filters logs per modules read the [Filtering section](./filtering.md).
