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

NOTE: for generic structs and enums the `derive` macro adds `Format` bounds to the *types of the generic fields* rather than to all the generic (input) parameters of the struct / enum.
Built-in `derive` attributes like `#[derive(Debug)]` use the latter approach.
To our knowledge `derive(Format)` approach is more accurate in that it doesn't over-constrain the generic type parameters.
The different between the two approaches is depicted below:

``` rust
# extern crate defmt;
# use defmt::Format;

#[derive(Format)]
struct S<'a, T> {
   x: Option<&'a T>,
   y: u8,
}
```

``` rust
# extern crate defmt;
# use defmt::Format;

// `Format` produces this implementation
impl<'a, T> Format for S<'a, T>
where
    Option<&'a T>: Format // <- main difference
{
    // ..
    # fn format(&self, f: &mut defmt::Formatter) {}
}

#[derive(Debug)]
struct S<'a, T> {
   x: Option<&'a T>,
   y: u8,
}
```

``` rust
# use std::fmt::Debug;
# struct S<'a, T> {
#    x: Option<&'a T>,
#    y: u8,
# }

// `Debug` produces this implementation
impl<'a, T> Debug for S<'a, T>
where
    T: Debug // <- main difference
{
    // ..
    # fn fmt(&self, f: &mut core::fmt::Formatter) -> std::fmt::Result { Ok(()) }
}
```
