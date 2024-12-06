# Bitfield

> ⚠️ The design and implementation chapter is outdated ⚠️

The integer argument is serialized in little endian format (`to_le_bytes`).

``` rust
# extern crate defmt;
# let x = 0u16;
defmt::error!("l: {0=0..8}, m: {0=8..12}, h: {0=12..16}", x /*: u16*/);
// on the wire: [1, 1, 2]
//  string index ^  ^^^^ `u16::to_le_bytes(x)`
```

Leading or trailing bytes that are not needed to display a bitfield are removed during serialization:

``` rust
# extern crate defmt;
defmt::error!("m: {0=8..12}", 0b0110_0011_0000_1111_u32);
// on the wire: [1, 0b0110_0011]
//  string index ^  ^^^^^^^^^^ argument truncated into u8:
//                             leading and trailing byte are irrelevant
```
