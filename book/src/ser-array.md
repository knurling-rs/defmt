# Arrays

> [!IMPORTANT]
> The design and implementation chapter is outdated

For arrays (`{=[u8; N]}`) the length is not serialized.

``` rust
# extern crate defmt;
defmt::error!("Data: {=[u8; 3]}!", [0, 1, 2]);
// on the wire: [1, 0, 1, 2]
//  string index ^  ^^^^^^^ the slice data
```
