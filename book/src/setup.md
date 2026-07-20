# Setup

To use `defmt` in a **library** you only need to add `defmt` as a dependency.

```console
$ cargo add defmt
```

To use `defmt` in an **application** you need the additional steps documented below.

## For applications

> 💡 The preferred way to create a *new* `defmt` application is to use our [app-template]. Tag along if you want to add `defmt` to an *existing* application.

[app-template]: https://github.com/knurling-rs/app-template

### Linker script

> 💡 This step only applies to *embedded* applications. Applications running on a host
> OS (Linux, macOS) don't need the linker script — see
> [Running on a host](#running-on-a-host-linux--macos) below.

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

> Note :
> If you intend on linking your project as a **static library** the linkerscript contents should be applied to the final binary.
> Adding the `-C link-arg=` option to static libraries has no effect because they are only archived, not linked.

### `#[global_logger]`

The application must link to or define a `global_logger`.
The `global_logger` specifies how logs are sent from the device running the app to the host where the logs are displayed.
The application must link to exactly one `global_logger`.
The `global_logger` can appear anywhere in the dependency graph and usually it will be its own crate.
The following `global_logger`s are provided as part of the project:

- [`defmt-rtt`], logs over RTT. Note that this crate can *not* be used together with `rtt-target`.
- [`defmt-itm`], logs over ITM (Instrumentation Trace Macrocell) stimulus port 0.
- [`defmt-semihosting`], logs over semihosting. Meant only for testing `defmt` on a virtual Cortex-M device (QEMU).
- `defmt-stdout`, logs to stdout (or a file), for programs and unit tests running on a host OS.

[`defmt-rtt`]: https://docs.rs/defmt-rtt/
[`defmt-itm`]: https://docs.rs/defmt-itm/
[`defmt-semihosting`]: https://github.com/knurling-rs/defmt/tree/6cfd947384debb18a4df761cbe454f8d86cf3441/firmware/defmt-semihosting

Information about how to write a `global_logger` can be found in the [`#[global_logger]` section](./global-logger.md).

### Enabling logging

By default, only ERROR level messages are logged.
To learn how to enable other logging levels and filter logs per module read the [Filtering section](./filtering.md).

### Memory use

When in a tight memory situation and logging over RTT, the buffer size (default: 1024 bytes) can be configured with the `DEFMT_RTT_BUFFER_SIZE` environment variable. Use a power of 2 for best performance.

## Running on a host (Linux / macOS)

`defmt` primarily targets embedded devices, but programs that use it — including your
crate's **unit tests** — can also be compiled for and run on a host operating system.
Both Linux (ELF) and macOS (Mach-O) executables are supported.

Host builds do *not* use the `defmt.x` linker script, so the
[linker script setup](#linker-script) above does not apply; only a `global_logger` is
needed. Use [`defmt-stdout`], which writes the binary defmt wire data to stdout, or to
a file if the `DEFMT_STDOUT_FILE` environment variable is set:

[`defmt-stdout`]: https://github.com/knurling-rs/defmt/tree/main/stdout

```toml
# Cargo.toml
[dependencies]
defmt = "1"
defmt-stdout = "0.1"
```

``` rust,ignore
// src/main.rs
use defmt_stdout as _;

fn main() {
    defmt::println!("Hello, x = {=u32}", 42);
}
```

`defmt-stdout` also provides the `_defmt_timestamp` and `_defmt_panic` symbols, whose
fallback implementations normally come from the linker script. If you define your own
with `defmt::timestamp!` or `#[defmt::panic_handler]`, disable the corresponding
default feature (`timestamp` / `panic-handler`).

Pipe the program's output through [`defmt-print`] to view the logs:

[`defmt-print`]: https://crates.io/crates/defmt-print

```console
$ DEFMT_LOG=info cargo run | defmt-print -e target/debug/my-app
Hello, x = 42
```

> 💡 Remember that log levels are selected at *compile time* with the `DEFMT_LOG`
> environment variable (see [Filtering](./filtering.md)). By default only ERROR level
> statements are compiled in.

### Running unit tests

Unit tests work the same way, with one caveat: `cargo test`'s own output ("running 1
test ...") goes to stdout too and would corrupt the binary defmt stream. Use
`DEFMT_STDOUT_FILE` to send the defmt data to a file instead:

```console
$ DEFMT_LOG=info DEFMT_STDOUT_FILE=defmt.bin cargo test
   Compiling my-app v0.1.0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.5s
     Running unittests src/main.rs (target/debug/deps/my_app-8759a844630659af)

running 1 test
test tests::logging_from_a_unit_test ... ok
```

Then decode the file, passing the *test binary* (the path is printed by `cargo test` on
the `Running` line) to `defmt-print`:

```console
$ defmt-print -e target/debug/deps/my_app-8759a844630659af < defmt.bin
INFO  hello from a unit test: 42
```

### Source location information on macOS (the dSYM trick)

`defmt-print` reads DWARF debug info to display file/line/module information (shown
with `--verbose`). On Linux the DWARF is embedded in the executable itself, so this
works out of the box.

On macOS, however, DWARF stays in the object files and is only collected into a
separate `.dSYM` bundle by the `dsymutil` tool. `defmt-print` looks for that bundle
next to the executable (e.g. `target/debug/my-app.dSYM`). To make Cargo generate it,
set [`split-debuginfo`] to `"packed"` in your profile:

```toml
# Cargo.toml
[profile.dev]
split-debuginfo = "packed"
```

Alternatively, run `dsymutil target/debug/my-app` by hand after building. This also
applies to test binaries: with `split-debuginfo = "packed"` Cargo places a `.dSYM`
bundle next to each test binary in `target/debug/deps/`.

> ⚠️ Only do this for macOS builds. On Linux, `split-debuginfo = "packed"` *removes*
> the DWARF from the executable (moving it into separate files that `defmt-print`
> doesn't read), so location information would be lost.

[`split-debuginfo`]: https://doc.rust-lang.org/cargo/reference/profiles.html#split-debuginfo