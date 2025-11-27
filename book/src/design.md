# Design & impl notes

Unstructured, braindump-ish notes about the design and implementation of `defmt`

> [!WARNING]
> the notes here may not accurately reflect the current implementation. This document is synchronized with the implementation at a *best effort* basis.

## Optimization goals

`defmt` optimizes for data throughput first and then for runtime cost.

## Constraints

### No double compilation

Say you want print logs from target/device app that uses crate `foo`.
That crate `foo` uses the `Format` trait on some of its data structures.
In this scenario we want to *avoid* having to compile `foo` for the host.
In other words, `foo` should only be (cross) compiled for the target device.

This is the biggest difference between `defmt` and some `serde` library that does binary serialization.
The `serde` library requires a `Deserialize` step that requires compiling `foo` for the host (see `derive(Serialize)`).
`defmt` avoids this by placing all the required information *for formatting* in a "side table" (see [the interning section](./interning.md)).

This comes with the downside that the host can only perform limited actions on the data it receives from the device: namely formatting.
The host cannot invoke `foo::Struct.method()` for example but that may not even be a sensible operation on the host anyways, e.g. `foo::USB::RegisterValue.store_volatile()` would make the host crash.

We want to avoid this "double" compilation (cross compile for the target *and* compile for the host) because:
- it doubles compilation time (wait time)
- compiling device-specific code for the host can become a headache quickly: see inline/external assembly, build scripts, etc.
