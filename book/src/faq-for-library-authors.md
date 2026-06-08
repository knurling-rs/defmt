# FAQ for library authors

You work on or maintain a general-purpose library and want to add `defmt` support?
Great!
This page will help you understand what `defmt` is and how to integrate it into your library.

## What is `defmt`?

`defmt` is a library to efficiently send logs from an embedded device to a host computer.
For more details, see [Introduction](./introduction.md).
It does this by separating compile-time information (format strings) from runtime information (values interpolated into format strings).
Types implement `Format` so defmt can encode their values.
If you want your library to support `defmt`, your public types should implement this trait.

## Should my library support `defmt`?

We recommend adding `defmt` support if your library:
- targets embedded users or supports `no_std`
- exposes types that implement `Debug` or would implement `Debug` if the crate was not aimed at embedded.

While `defmt`'s main goal is to enable efficient logging in an embedded context, support for higher level platforms and standard library support is likely in the future.

## Does `Format` replace `Debug` or `Display`?

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

Depending on your codebase, you may want to use one of the following patterns:

```rust
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct MyStruct { .. }

#[cfg(feature = "defmt")]
impl defmt::Format for MyStruct { .. }

#[cfg(feature = "defmt")]
mod defmt;
```

## Which `defmt` version should I depend on?

For new integrations, depend on the latest stable major version of `defmt`.
Avoid pinning exact patch versions unless you need to work around a known issue.

## How should I format third-party types?

If the type you want to implement `Format` for contains types controlled by a third party, you have several options.

1. If the third-party crate also has `defmt` support, consider making your `defmt` feature enable its `defmt` feature.
1. If they don't support `defmt` yet, consider asking them to add it.
1. Write a [`Format`](./format.md#manual-implementation-with-write) implementation and print information retrieved from the inner type manually.
1. Use one of the [uncompressed adapters](./format.md#uncompressed-adapters);
   Only recommended as a last resort since these bypass the efficiency features of `defmt`.

## How can I test `defmt` support?

For most libraries implementing `defmt`, it should be sufficient to test whether compilation is successful with the `defmt` feature enabled.
This is especially true if you don't have any manual `Format` implementations.

```sh
cargo check --features defmt
```

### Runtime smoke tests

If your `Format` implementation calls helper methods, test those individually.

Alternatively, you can call `defmt::println!` on the type in your test.
In that case make sure to register a dummy [global logger](./global-logger.md) to avoid linker errors.

```sh
cargo test --features defmt
```

### Testing exact output

If you want to ensure your custom `Format` implementations produce the exact strings you intend, one option is to use [qemu-run](https://crates.io/crates/qemu-run) to execute a cross-compiled binary through [QEMU](https://www.qemu.org/).
Note that this is usually not necessary unless you consider exact formatted log strings part of your public API.
