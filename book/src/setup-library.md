# Library setup

If your library will use any of the logging macros (`trace!`, `debug!`, `info!`, `warn!`, `error!`) then you'll need to add these Cargo features to your library's `Cargo.toml`:

``` toml
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

You do not need to add these features if you are only going to use the `#[derive(Format)]` attribute.
