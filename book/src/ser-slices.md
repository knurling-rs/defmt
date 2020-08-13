# Slices

For slices (`{:[u8]}`) the length is LEB128 encoded and serialized first and then followed by the slice data.

``` rust
# extern crate binfmt;
binfmt::error!("Data: {:[u8]}!", [0, 1, 2]);
// on the wire: [1, 3, 0, 1, 2]
//  string index ^  ^  ^^^^^^^ the slice data
//   LEB128(length) ^
```
