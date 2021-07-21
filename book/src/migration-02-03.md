# Migrating from `v0.2.x` to `v0.3.0`

This guide covers how to upgrade a library or application using `defmt v0.2.x` to version `v0.3.0`.

## `Cargo.toml`

Update the version of `defmt` to `"0.3"` (or `"0.3.0"`, which is equivalent).

Additionally please remove the `defmt-*` cargo features from your `[features]` section.

```diff
[dependencies]

- defmt = "0.2"
+ defmt = "0.3"

[features]
other-feature = []

- defmt-default = []
- defmt-trace = []
- defmt-debug = []
- defmt-info = []
- defmt-warn = []
- defmt-error = []
```

## `DEFMT_LOG`

Setting the log-level via cargo features is superseded by the new `DEFMT_LOG` environment variable.

To log everything on the `INFO` level and above, run your application like following:

```console
$ DEFMT_LOG=info cargo run
```

For more details how to configure the log-level using `DEFMT_LOG` see the [user docs](TODO: add link).

---

TODO

- [ ] `#505`: Logger trait v2
- [ ] `#521`: [3/n] Remove u24
- [ ] `#522`: Replace `µs` hint with `us`
- [ ] `#508`: [5/n] Format trait v2
  - no Write trait anymore
- [x] `#519`: `DEFMT_LOG`

