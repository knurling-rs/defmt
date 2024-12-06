# Strings

> ⚠️ The design and implementation chapter is outdated ⚠️

Strings that are passed directly (i.e. not as indices of interned strings) as format string parameters (`{:str}`) must be prefixed with their LEB128 encoded length.
This behavior is analogous to that of Slices.

``` rust
# extern crate defmt;
defmt::error!("Hello, {=str}!", "world");
// on the wire: [1, 5, 199, 111, 114, 108, 100]
//  string index ^  ^  ^^^^^^^^^^^^^^^^^^^^^^^ the slice data
//   LEB128(length) ^
```
