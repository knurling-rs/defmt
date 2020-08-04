# Bitfield

The integer argument is serialized in little endian format (`to_le_bytes`).

``` rust
# extern crate binfmt;
# let x = 0u16;
binfmt::error!("l: {0:0..8}, m: {0:8..12}, h: {0:12..16}", x /*: u16*/);
// on the wire: [1, 1, 2]
//  string index ^  ^^^^ `u16::to_le_bytes(x)`
```
