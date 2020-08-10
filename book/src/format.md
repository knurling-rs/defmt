# `#[derive(Format)]`

To implement the `Format` trait for a struct or enum use the `derive` attribute.

``` rust
#[derive(Format)]
struct Header {
    source: u8,
    destination: u8,
    sequence: u16,
}

#[derive(Format)]
enum Request {
    GetDescriptor { descriptor: Descriptor, length: u16 },
    SetAddress { address: u8 },
}
```
