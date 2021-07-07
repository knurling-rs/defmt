# Filtering

`defmt` supports 5 different logging levels listed below from lowest severity to highest severity:

- `TRACE`
- `DEBUG`
- `INFO`
- `WARN`
- `ERROR`

By default all logging is *disabled*.
The amount of logging to perform can be controlled at the *crate* level using the following Cargo features:

- `defmt-trace`, log at `TRACE` level and up
- `defmt-debug`, log at `DEBUG` level and up
- `defmt-info`, log at `INFO` level and up
- `defmt-warn`, log at `WARN` level and up
- `defmt-error`, log at `ERROR` level
- `defmt-default`, log at `INFO`, or `TRACE`, level and up

These features must only be enabled by the top level *application* crate as shown below.

``` toml
# Cargo.toml
[package]
name = "app"

[dependencies]
usb-device = "0.3.0"

[features]
default = [
  # enable logs of the `usb-device` dependency at the TRACE/INFO level
  "usb-device/defmt-default",

  # enable logs of this crate (`app`) at the TRACE level
  "defmt-trace",
]
```

When only the `defmt-default` feature is enabled for a crate, that crate will:

- log at the TRACE level and up if `debug-assertions = true` (`dev` profile), or
- log at the INFO level and up if `debug-assertions = false` (`release` profile)

When any of the other features is enabled the crate will log at that, and higher, severity regardless of the state of `debug-assertions` or `defmt-default`.
