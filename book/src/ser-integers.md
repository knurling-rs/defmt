# Integers

Integers will be serialized in little endian order using `to_le_bytes()`.
`usize` and `isize` values will be subject to LEB128 compression.

``` rust
# extern crate binfmt;
binfmt::error!("The answer is {:i16}!", 300);
// on the wire: [3, 44, 1]
//  string index ^  ^^^^^ `300.to_le_bytes()`
//  ^ = intern("The answer is {:i16}!")

binfmt::error!("The answer is {:u24}!", 131000);
// on the wire: [4, 184, 255, 1]
//                  ^^^^^^^^^^^ 131000.to_le_bytes()[..3]

binfmt::error!("The answer is {:usize}!", 131000);
// on the wire: [4, 184, 255, 1]
//                  ^^^^^^^^^^^ 131000.to_le_bytes()[..3]
```

> NOTE(japaric) unclear to me if LEB128 encoding (more compression but more) `u16` and `u32` is worth the trade-off

> TODO(japaric) evaluate [zigzag encoding][zigzag] for `isize`?

[zigzag]: https://developers.google.com/protocol-buffers/docs/encoding
