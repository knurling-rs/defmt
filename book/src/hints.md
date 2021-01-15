# Display hints

A hint can be applied to each formatting parameter to change how it's printed on the host side.
The hint follows the syntax `:x` and must come after the type within the braces.
Examples: typed `{=u8:x}`, untyped `{:b}`

The following display hints are currently supported:

- `x`, lowercase hexadecimal
- `X`, uppercase hexadecimal
- `?`, `core::fmt::Debug`-like
- `b`, binary
- `a`, ASCII
- `µs`, microseconds (formats integers as time stamps)

The first 4 display hints resemble what's supported in `core::fmt`. Examples below:

``` rust
# extern crate defmt;
defmt::info!("{=u8:x}", 42); // -> INFO 0x2a
defmt::info!("{=u8:X}", 42); // -> INFO 0x2A
defmt::info!("{=u8:b}", 42); // -> INFO 0b101010

defmt::info!("{=str}", "hello\tworld");   // -> INFO hello    world
defmt::info!("{=str:?}", "hello\tworld"); // -> INFO "hello\tworld"
```

No further customization like padding, leading zeroes is supported (at the moment).

The ASCII display hint formats byte slices (and arrays) using Rust's byte string syntax.

``` rust
# extern crate defmt;
let bytes = [104, 101, 255, 108, 108, 111];

defmt::info!("{=[u8]:a}", bytes);
// -> INFO b"he\xffllo"
```

## Propagation

Display hints "propagate downwards" and apply to formatting parameters that specify no display hint.

``` rust
# extern crate defmt;
#[derive(defmt::Format)]
struct S { x: u8 }

let x = S { x: 42 };
defmt::info!("{}", x);
// -> INFO S { x: 42 }

defmt::info!("{:x}", x);
// -> INFO S { x: 0x2a }
```

``` rust
# extern crate defmt;
struct S { x: u8, y: u8 }

impl defmt::Format for S {
    fn format(&self, f: defmt::Formatter) {
        // `y`'s display hint cannot be overriden (see below)
        defmt::write!(f, "S {{ x: {=u8}, y: {=u8:x} }}", self.x, self.y)
    }
}

let x = S { x: 42, y: 42 };
defmt::info!("{}", x);
// -> INFO S { x: 42, y: 0x2a }

defmt::info!("{:x}", x);
// -> INFO S { x: 0b101010, y: 0x2a }
```
