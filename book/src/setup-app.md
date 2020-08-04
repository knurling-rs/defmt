# Application setup

## Cargo features

Add these Cargo features to your app's `Cargo.toml`:

``` toml
# Cargo.toml
# under the features section, copy these
[features]
# ↓↓↓↓↓
binfmt-default = []
binfmt-trace = []
binfmt-debug = []
binfmt-info = []
binfmt-warn = []
binfmt-error = []
# ↑↑↑↑↑
```

## Linker script

The application must be linked using a custom linking process that includes the `binfmt.x` linker script.
Custom linking is usual for embedded applications and configured in the `.cargo/config` file.
To pass `binfmt.x` to the linker add the `-C link-arg=-Tbinfmt.x` flag to the rustflags section of `.cargo/config`.

``` toml
# .cargo/config
[target.thumbv7m-none-eabi]
rustflags = [
  # likely, there's another `link-arg` flag already there; KEEP it
  "-C", "link-arg=-Tlink.x",
  "-C", "link-arg=-Tbinfmt.x", # <- ADD this one
]
```

## `#[global_logger]`

The application must link to or define a `global_logger`.
The `global_logger` specifies how logs are sent from the device running the app to the host where the logs are displayed.
The application must link to exactly one `global_logger`.
The `global_logger` can appear anywhere in the dependency graph and usually it will be its own crate.
The following `global_logger`s are provided as part of the project:

- `binfmt-semihosting`, logs over semihosting. Meant only for testing `binfmt` on a virtual Cortex-M device (QEMU).
- `binfmt-rtt`, logs over RTT. Note that this crate can *not* be used together with `rtt-target`.

Information about how to write a `global_logger` can be found in the [`#[global_logger]` section](./global-logger.md).

## `#[timestamp]`

The application must link to or define a `timestamp` function.
All logs are timestamped; this function specifies how the timestamp is computed.
The function must have signature `fn() -> u64`; the returned value is the timestamp in microseconds (`0` = program started).
The function should be implemented using a non-decreasing hardware counter but here are two pure-software implementations that can be used as a placeholder:

No timestamps:

``` rust
#[binfmt::timestamp]
fn no_timestamp() -> u64 {
    0
}
```

Virtual timestamps:

``` rust
// WARNING may overflow and wrap-around in long lived apps
#[binfmt::timestamp]
fn virtual_timestamp() -> u64 {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    COUNT.fetch_add(1, Ordering::Relaxed) as u64
}
```

More information about how to write a `timestamp` function can be found in the [`#[timestamp]` section](./timestamp.md).

## Enabling logging

All logging is *disabled* by default.
Logging can be enabled at the *crate* level.
At the very least you'll want to enable logging for the top level application crate so we recommend adding `binfmt-default` to your crate's `default` feature.

``` toml
# Cargo.toml
[features]
default = [
  "binfmt-default", # <- ADD this feature
]
```

More information about log filtering can be found in the [Filtering section](./filtering.md).
