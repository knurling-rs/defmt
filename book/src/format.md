# Implementing `Format`

## `#[derive(Format)]`

The easiest way to implement the `Format` trait for a struct or enum is to use the `derive` attribute.

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
#[derive(Format)]
enum Request {
    GetDescriptor { descriptor: Descriptor, length: u16 },
    SetAddress { address: u8 },
}
```

NOTE: Like built-in derives like `#[derive(Debug)]`, `#[derive(Format)]` will add `Format` bounds to the generic type parameters of the struct.

NOTE: Do *not* use the API used by the expansion of the `derive(Format)` macro; it is *unstable*.

## `write!`

> NOTE `write!` is available in `defmt` v0.1.**2**+

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
           "CRCCNF {{ LEN: {0:0..2}, SKIPADDR: {0:8..10} }}",
           self.bits,
        )
    }
}
```

NOTE: in defmt v0.1.x the `write!` macro must be invoked at most once within the implementation of the `fmt` method. Invoking the method more than once will produce a panic

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

**WARNING** never call `format` more than once, since this will corrupt the data stream!
