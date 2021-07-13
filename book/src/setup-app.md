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

  # --- ADD following new flags ---
  "-C", "link-arg=-Tdefmt.x",
  # This is needed if your flash or ram addresses are not aligned to 0x10000 in memory.x
  # See https://github.com/rust-embedded/cortex-m-quickstart/pull/95
  "-C", "link-arg=--nmagic",
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
[`defmt-itm`]: https://docs.rs/defmt-rtt/
[`defmt-semihosting`]: https://github.com/knurling-rs/defmt/tree/6cfd947384debb18a4df761cbe454f8d86cf3441/firmware/defmt-semihosting

Information about how to write a `global_logger` can be found in the [`#[global_logger]` section](./global-logger.md).

## Enabling logging

All logging is *disabled* by default.
Logging can be enabled at the *crate* level.
At the very least you'll want to enable logging for the top level application crate so we recommend adding `defmt-default` to your crate's `default` feature.

``` toml
# Cargo.toml
[features]
default = [
  "defmt-default", # <- ADD this feature
]
```

More information about log filtering can be found in the [Filtering section](./filtering.md).
