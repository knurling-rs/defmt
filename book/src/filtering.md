# Filtering

`defmt` supports 5 different logging levels listed below from lowest severity to highest severity:

- `TRACE`
- `DEBUG`
- `INFO`
- `WARN`
- `ERROR`

The logging macros `trace!`, `debug!`, `info!`, `warn!` and `error!` match these logging levels.

By default, only `ERROR` level messages are emitted.
All other logging levels are disabled.

Note that `defmt::println!` statements cannot be filtered and are always included in the output.

## `DEFMT_LOG`

> if you are already familiar with [`env_logger`] and `RUST_LOG`, `defmt`'s filtering mechanism works very similarly

[`env_logger`]: https://docs.rs/env_logger/0.9.0/env_logger/

To change which logging levels are enabled use the `DEFMT_LOG` environment variable.

``` console
$ export DEFMT_LOG=warn
$ cargo build --bin app

$ # OR if using probe-run as the cargo runner you can use
$ DEFMT_LOG=warn cargo run --bin app
```

`DEFMT_LOG` accepts the following logging levels: `error`, `warn`, `info`, `debug`, `trace`.
Enabling a logging level also enables higher severity logging levels.
For example,

``` rust,ignore
defmt::trace!("trace");
defmt::debug!("debug");
defmt::info!("info");
defmt::warn!("warn");
defmt::error!("error");
```

``` console
$ DEFMT_LOG=warn cargo run --bin all-logging-levels
WARN  warn
ERROR error

$ DEFMT_LOG=trace cargo run --bin all-logging-levels
TRACE trace
DEBUG debug
INFO  info
WARN  warn
ERROR error
```

## Modules

A different logging level filter can be applied to different modules using *logging directives*.
A logging directive has the syntax `krate::module::path=level`.
`DEFMT_LOG` can contain a list of comma separated logging directives.

``` rust,ignore
// crate-name = app

mod important {
    pub fn function() {
        defmt::debug!("important debug");
        defmt::info!("important info");
        defmt::warn!("important warn");
        defmt::error!("important error");
    }
}

mod noisy {
    pub fn function() {
        defmt::warn!("noisy warn");
        defmt::error!("noisy error");
        inner::function();
    }

    mod inner {
        pub fn function() {
            defmt::warn!("inner warn");
            defmt::error!("inner error");
        }
    }
}

important::function();
noisy::function();
```

``` console
$ DEFMT_LOG=app::important=info,app::noisy=error cargo run --bin app
INFO  important info
WARN  important warn
ERROR important error
ERROR noisy error
ERROR inner error
```

Note that the `app::noisy=error` directive also applies to the internal module `app::noisy::inner`.

### Hyphens

Crate names can have hyphens (`-`) in Cargo metadata, and file paths, but when they appear in logging directives all hyphens must be converted into underscores (`_`).

### Packages vs crates

Do not confuse Cargo package names with crate names.
A Cargo package can contain more than one crate.
By default, the main crate has the same name as the package but this can be overridden in `Cargo.toml` (e.g. in the `[lib]` and `[[bin]]` sections).

``` console
$ cargo new --lib my-package

$ tree my-package
my-package # package-name = my-package
├── Cargo.toml
└── src
   ├── bin
   │  └── my-binary.rs # crate-name = my_binary
   └── lib.rs          # crate-name = my_package
```

## Overrides

Logging directives that appear later in the list override preceding instances.

``` rust,ignore
// crate-name = app
pub fn function() {
    defmt::trace!("root trace");
}

mod inner {
    pub fn function() {
        defmt::trace!("inner trace");
        defmt::error!("inner error");
    }
}

function();
inner::function();
```

``` console
$ DEFMT_LOG=trace,app::inner=error cargo run --bin app
TRACE root trace
ERROR inner error
```

This is equivalent to saying:
`app::inner` emits ERROR level log messages and everything else emits TRACE level log messages.

## Disabling logs

The "pseudo" logging level `off` can be used to disable logs globally, per crate or per module.

``` console
$ # globally disable logs
$ DEFMT_LOG=off cargo run --bin app

$ # disable logs from the `noisy` crate (dependency)
$ DEFMT_LOG=trace,noisy=off cargo run --bin app

$ # disable logs from the `noisy` module
$ DEFMT_LOG=trace,app::noisy=off cargo run --bin app
```

## Recompilation

It should be noted that `DEFMT_LOG` is a *compile-time* mechanism.
Changing the contents of `DEFMT_LOG` will cause all crates that depend on `defmt` to be recompiled.

## Default logging level for a crate

At the moment it's **not** possible to set a default logging level, other than ERROR, for a crate.
