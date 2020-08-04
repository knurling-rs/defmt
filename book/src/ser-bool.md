# Bool


Booleans are compressed in bytes, bitflags-style.

``` rust
binfmt::error!("x: {:bool}, y: {:bool}, z: {:bool}", false, false, true);
// on the wire: [1, 0b001]
//  string index ^  ^^^^^ the booleans: `0bxyz`
```

When mixed with other data, the first `{:bool}` allocates an output byte that
fits up to 7 more bools.

`{:bool}`s in the formatting string are batched together as follows: Any non-`{:bool}` arguments are emitted as-is, while `{:bool}` arguments are collected into a byte and emitted when 8 `{:bool}`s have been collected. This means that for every set of 8 `{:bool}`s, the byte containing them will be serialized at the position of their last member.
If more than 0 but less than 8 `{:bool}`s have been encountered at the end of the log frame, a byte containing them will be emitted last.

``` rust
binfmt::error!("x: {:bool}, y: {:u8}, z: {:bool}", false, 0xff, true);
// on the wire: [1, 0xff, 0b01]
//  string index ^  ^^^^^ ^^^^ the booleans: `0bxz`
//                  |
//                  u8
```

⚠️ If the final parameter is not a `{:bool}` but there are yet to be compressed `{:bool}`s present in the format string beforehand, the final output byte containing all compressed booleans will be at the end.

``` rust
binfmt::error!("x: {:bool}, y: {:u8}", false, 0xff);
// on the wire: [1, 0xff, 0b0,]
//  string index ^  ^^^^^ ^^^^ the booleans: `0bx`
//                  |
//                  u8
```

⚠️ If some `{:bool}`s are nested inside a struct, they will still be compressed as if they were passed as regular arguments.

``` rust
struct Flags {
         a: bool,
         b: bool,
}

binfmt::error!("x: {:bool}, {:?}", false, Flags { a: true, b: false });
// on the wire: [1, 2, 0b010,]
//  string index ^  ^  ^^^^ all booleans: `0bxab`
//                  |
//                  index of "Flags { a: {:bool}, b: {:bool}} "
```
