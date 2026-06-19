# FAQ for library authors

You work on or maintain a general-purpose library and want to add `defmt` support?
Great!
This page will help you understand what `defmt` is and how to integrate it into your library.

## What is `defmt`?

`defmt` is a logging framework for deferred formatting on resource-constrained embedded devices.
It provides a familiar logging API while staying cheap enough for microcontrollers.

`defmt` gets its efficiency from:
- deferred string interpolation: the program on the microcontroller sends compact binary data, and the host formats it later.
- string interning: format strings don't end up on the microcontroller and are not sent with each log frame.
- compact encoding and compression: useful logs can fit through bandwidth-constrained channels.
- no allocations: formatting does not need runtime allocation on the target.
- no reliance on `core::fmt`: `Format` implementations encode values directly, avoiding expensive string formatting on the target and reducing both runtime overhead and flash usage.

For more details, see [Introduction](./introduction.md).

Types implement `Format` so `defmt` can encode their values.
If you want your library to support `defmt`, public types that users may log should implement this trait.

## Should my library support `defmt`?

We recommend adding `defmt` support if your library:
- targets embedded users or supports `no_std`
- exposes types that users commonly inspect in logs or debug output.

## How does `Format` compare to `Debug` or `Display`?

- [core::fmt::Display](https://doc.rust-lang.org/core/fmt/trait.Display.html)

  This trait is intended to generate a user-facing string from a type.
  It usually does not contain structural parts like `{ }` or exact field names.
  Only implement this if your type has a "nice" string representation.

- [core::fmt::Debug](https://doc.rust-lang.org/core/fmt/trait.Debug.html)

  This trait is commonly used for debugging and logs; the result is usually not shown to an end user.
  It should render all information necessary to understand the state of your type.
  This includes recursing on member field types.

- [defmt::Format](https://docs.rs/defmt/latest/defmt/trait.Format.html)

  `Format` should behave similarly to `Debug` but does not replace it.
  The goal is to provide a representation of the state of your type that is as complete as reasonably possible.

  The main difference is where the printable string is generated.
  `Debug` generates it immediately whereas `Format` only encodes the format string index and argument values.
  Construction of the interpolated string then happens on the [printer](./printers.md) side, usually a host computer.

## How do I implement `Format`?

See the [Implementing Format](./format.md) page.

## When should I use `defmt::write!` instead of `format!`?

Use `defmt::write!` inside manual `Format` implementations.
It writes into the `defmt` formatter and preserves `defmt`'s compact encoding.

Do not use `format!` to build a `String` for `defmt` output!
`format!` uses Rust formatting, allocates, and loses the encoding benefits of `defmt`.

## How should I depend on `defmt`?

For general-purpose libraries, add `defmt` as an optional dependency and gate all `Format` implementations behind a `defmt` feature.
Do not make `defmt` a required dependency unless your crate is specifically a `defmt` integration crate.

```toml
[dependencies]
defmt = { version = "1", optional = true }

[features]
defmt = ["dep:defmt"]
```

Depending on your codebase, we suggest using the following patterns:

```rust
// For feature gating derives
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct MyStruct {  }

// For manual implementations
#[cfg(feature = "defmt")]
impl defmt::Format for MyStruct {  }

// To reduce feature-gate clutter, group defmt-specific implementations
# struct MyOtherStruct {  }
#
#[cfg(feature = "defmt")]
mod defmt {
    impl defmt::Format for MyStruct {  }
    impl defmt::Format for MyOtherStruct {  }
}
```

## Which `defmt` version should I depend on?

For new integrations, depend on the latest stable major version of `defmt`.
Avoid pinning exact patch versions unless you need to work around a known issue.

## How should I format third-party types?

If the type you want to implement `Format` for contains types controlled by a third party, you have several options.

1. If the third-party crate also has `defmt` support, consider making your `defmt` feature enable its `defmt` feature.
1. If they don't support `defmt` yet, consider asking them to add it.
1. Write a [`Format`](./format.md#manual-implementation-with-write) implementation and print information retrieved from the inner type manually.
1. Use one of the [uncompressed adapters](./format.md#uncompressed-adapters); use as a last resort since these bypass `defmt`'s efficiency features.

## How can I test my `defmt` support?

For most libraries, compile-checking with the `defmt` feature enabled is enough.
This is especially true if you only derive `Format`.

Because `defmt` support may not compile for your host system, test with an embedded target:

```sh
cargo check --target thumbv7m-none-eabi --features defmt
```

If you write manual `Format` implementations, keep non-formatting logic in normal helper functions and unit-test those separately.

Testing exact formatted output usually is not necessary unless you treat that output as part of your public API.
If you need it, run a small cross-compiled binary through [QEMU](https://www.qemu.org/), for example with [`qemu-run`](https://crates.io/crates/qemu-run).
