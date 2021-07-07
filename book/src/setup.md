# Setup

Before you use the `defmt` crate some setup may be necessary.
There is some base setup needed for both libraries and applications and some additional steps only for applications.

> 💡 The prefferred way to create a *new* `defmt` application is to use our [app-template]. Tag along if you want to add `defmt` to an *existing* appliaction.

[app-template]: https://github.com/knurling-rs/app-template

## Base setup

If you only use the `#[derive(Format)]` attribute and no logging macros then you do not need do anything except adding `defmt` to your dependencies.

```console
$ cargo add defmt
```

If your library/application will use any of the logging macros (`trace!`, `debug!`, `info!`, `warn!`, `error!`), which it will likely do, then you will need to add following Cargo features to your `Cargo.toml`:

``` toml
[features]
existing-feature = []

# --- ADD THOSE ---
defmt-default = []
defmt-trace = []
defmt-debug = []
defmt-info = []
defmt-warn = []
defmt-error = []
```

---

That is already it for setting up `defmt` for your library!
See the next chapter for additional steps for applications.
