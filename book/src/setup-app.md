# Application setup

**NOTE** the preferred way to create a *new* `defmt` application is to use our [`app-template`].
These steps are for using `defmt` on an *existing* application.

[`app-template`]: https://github.com/knurling-rs/app-template

## Cargo features

Add these Cargo features to your app's `Cargo.toml`:

``` toml
# Cargo.toml
# under the features section, copy these
[features]
# ↓↓↓↓↓
defmt-default = []
defmt-trace = []
defmt-debug = []
defmt-info = []
defmt-warn = []
defmt-error = []
# ↑↑↑↑↑
```

## Linker script

The application must be linked using a custom linking process that includes the `defmt.x` linker script.
Custom linking is usual for embedded applications and configured in the `.cargo/config` file.
To pass `defmt.x` to the linker add the `-C link-arg=-Tdefmt.x` flag to the rustflags section of `.cargo/config`.

``` toml
# .cargo/config
[target.thumbv7m-none-eabi]
rustflags = [
  # likely, there's another `link-arg` flag already there; KEEP it
  "-C", "link-arg=-Tlink.x",
  "-C", "link-arg=-Tdefmt.x", # <- ADD this one
]
```

## `#[global_logger]`

The application must link to or define a `global_logger`.
The `global_logger` specifies how logs are sent from the device running the app to the host where the logs are displayed.
The application must link to exactly one `global_logger`.
The `global_logger` can appear anywhere in the dependency graph and usually it will be its own crate.
The following `global_logger`s are provided as part of the project:

- [`defmt-rtt`], logs over RTT. Note that this crate can *not* be used together with `rtt-target`.
- [`defmt-semihosting`], logs over semihosting. Meant only for testing `defmt` on a virtual Cortex-M device (QEMU).

[`defmt-semihosting`]: https://github.com/knurling-rs/defmt/tree/9f97c1fd562738159a142bd67c410c48ef8d4110/firmware/defmt-semihosting
[`defmt-rtt`]: https://docs.rs/defmt-rtt/

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
