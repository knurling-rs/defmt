# Implementing `Format`

## `#[derive(Format)]`

The easiest way to implement the `Format` trait for a `struct` or `enum` is to use the `derive` attribute.

``` rust
# extern crate defmt;
use defmt::Format;

#[derive(Format)]
struct Header {
    source: u8,
    destination: u8,
    sequence: u16,
}

# #[derive(Format)]
# struct Descriptor;
#
#[derive(Format)]
enum Request {
    GetDescriptor { descriptor: Descriptor, length: u16 },
    SetAddress { address: u8 },
}
```

Like built-in derives (e.g. `#[derive(Debug)]`), `#[derive(Format)]` will add `Format` bounds to the generic type parameters of the struct.

> ⚠️ Do *not* use the API used by the expansion of the `derive(Format)` macro; it is *unstable*.

By default the derive assumes `defmt` to exist in your crate's dependencies (extern crate prelude).
If that is not the case you can overwrite the crate path the derive should use in its expansion via the `defmt(crate = path)` helper attribute.

The alternative `defmt` crate path needs to be specified for every `derive(other_defmt::Format)` macro.

```rust
extern crate defmt as other_defmt;

#[derive(other_defmt::Format)]
#[defmt(crate = other_defmt)]
struct Header;
```

## Feature-gated `#[derive(Format)]`

It is also possible to feature-gate the implementation by defining
`derive` implementation via `cfg_attr(feature = ...)`:

```rust
#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct Header {
    // ...
}
```

## Manual implementation with `write!`

It is also possible to implement the `Format` trait manually.
This trait has a single required method: `format`.
In this method you need to format the value (`self`) into the given `Formatter` argument using the `defmt::write!` macro.
Example below:

``` rust
# extern crate defmt;
// value read from a MMIO register named "CRCCNF"
struct CRCCNF {
   bits: u32,
}

impl defmt::Format for CRCCNF {
    fn format(&self, f: defmt::Formatter) {
        // format the bitfields of the register as struct fields
        defmt::write!(
           f,
           "CRCCNF {{ LEN: {0=0..2}, SKIPADDR: {0=8..10} }}",
           self.bits,
        )
    }
}
```

## Newtypes

If you need to implement `Format` for some "newtype" struct you can delegate the formatting to the inner type.
Example below:

``` rust
# extern crate defmt;
struct MyU8 { inner: u8 }

impl defmt::Format for MyU8 {
    fn format(&self, f: defmt::Formatter) {
        self.inner.format(f)
    }
}
```

## Uncompressed adapters

If you quickly want to get some code running and do not care about it being efficient you can use the two adapter types [`Display2Format`] and [`Debug2Format`].

> ⚠️ These adapters disable compression and use the `core::fmt` code on-device! You should always prefer `defmt::Format` over `Debug` whenever possible!

Note that this always uses `{:?}` to format the contained value, meaning that any provided defmt display hints will be ignored.

When using `#[derive(Format)]` you may use the `#[defmt()]` attribute on specific fields to use these adapter types.
Example below:

``` rust
# extern crate defmt;
# use defmt::Format;
# mod serde_json {
#     #[derive(Debug)]
#     pub enum Error {}
# }
#[derive(Format)]
enum Error {
    Serde(#[defmt(Debug2Format)] serde_json::Error),
    ResponseTooLarge,
}

# struct Celsius();
# impl std::fmt::Display for Celsius {
#     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { Ok(()) }
# }
#[derive(Format)]
struct Reading {
    #[defmt(Display2Format)]
    temperature: Celsius,
}
```

[`Display2Format`]: https://docs.rs/defmt/*/defmt/struct.Display2Format.html
[`Debug2Format`]: https://docs.rs/defmt/*/defmt/struct.Debug2Format.html
