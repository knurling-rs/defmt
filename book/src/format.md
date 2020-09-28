# `#[derive(Format)]`

To implement the `Format` trait for a struct or enum use the `derive` attribute.

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
