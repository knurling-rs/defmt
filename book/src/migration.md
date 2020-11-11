# Migrating from git defmt to stable defmt

On 2020-11-11, a stable version of `defmt` became available on crates.io.
If you are still using the git version you are encouraged to migrate your project to the crates.io version!
Two things need to be done to use the crates.io version of `defmt`:

1. change `defmt`, `defmt-rtt` and `panic-probe` dependencies from git to version `"0.1.0"` in relevant `Cargo.toml`-s
2. install version v0.1.4 (or newer) of [`probe-run`]

[`probe-run`]: https://github.com/knurling-rs/probe-run

Here's are the exact steps for migrating an [`app-template`] project.

[`app-template`]: https://github.com/knurling-rs/app-template

1. In your `app-template` project, change the root `Cargo.toml` as shown below:

``` diff
 [workspace]
 members = ["testsuite"]

-[dependencies.defmt]
-git = "https://github.com/knurling-rs/defmt"
-branch = "main"
-
-[dependencies.defmt-rtt]
-git = "https://github.com/knurling-rs/defmt"
-branch = "main"
-
-[dependencies.panic-probe]
-git = "https://github.com/knurling-rs/probe-run"
-branch = "main"
-
 [dependencies]
+defmt = "0.1.0"
+defmt-rtt = "0.1.0"
+panic-probe = { version = "0.1.0", features = ["print-defmt"] }
 cortex-m = "0.6.4"
 cortex-m-rt = "0.6.13"
```

2. In your `app-template` project, also change the `testsuite/Cargo.toml` as shown below:

``` diff
 name = "test"
 harness = false

-[dependencies.defmt]
-git = "https://github.com/knurling-rs/defmt"
-branch = "main"
-
-[dependencies.defmt-rtt]
-git = "https://github.com/knurling-rs/defmt"
-branch = "main"
-
-[dependencies.panic-probe]
-git = "https://github.com/knurling-rs/probe-run"
-branch = "main"
-# enable the `print-defmt` feature for more complete test output
-features = ["print-defmt"]
-
 [dependencies]
+defmt = "0.1.0"
+defmt-rtt = "0.1.0"
+panic-probe = { version = "0.1.0", features = ["print-defmt"] }
 cortex-m = "0.6.3"
 cortex-m-rt = "0.6.12"
```

3. Finally, install `probe-run` version v0.1.4 (or newer)

``` console
$ cargo install probe-run -f
```

Now you can resume working on your project!

<!-- TODO(japaric) check if `cargo clean` or `cargo update` is needed after step 3 -->
