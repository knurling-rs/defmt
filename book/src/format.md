# `#[derive(Format)]`

To implement the `Format` trait for a struct or enum use the `derive` attribute.

<!-- TODO remove ',ignore' -->
``` rust,ignore
# extern crate binfmt;
use binfmt::Format;

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
