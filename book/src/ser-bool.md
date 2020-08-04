# Bool

Booleans are grouped in bytes, bitflags-style.

``` rust
binfmt::error!("x: {:bool}, y: {:bool}, z: {:bool}", false, false, true);
// on the wire: [1, 0b100]
//  string index ^  ^^^^^ the booleans: `0bzyx`
```

When mixed with other data, the first `{:bool}` allocates an output byte that
fits up to 7 more bools.

``` rust
binfmt::error!("x: {:bool}, y: {:u8}, z: {:bool}", false, 0xff, true);
// on the wire: [1, 0b10, 0xff]
//  string index ^  ^^^^^ ^^^^ u8
//                  |
//                  the booleans: `0bzx`
```
