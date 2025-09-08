# Display hints

> A hint can be applied to each formatting parameter to change how it's printed on the host side.

The hint follows the syntax `:Display` and must come after the type within the braces, for example:
* typed `{=u8:x}`
* untyped `{:b}`

The following display hints are currently supported:

| hint    | name                                                     |
| :------ | :------------------------------------------------------- |
| `:x`    | lowercase hexadecimal                                    |
| `:X`    | uppercase hexadecimal                                    |
| `:?`    | `core::fmt::Debug`-like                                  |
| `:b`    | binary                                                   |
| `:o`    | octal                                                    |
| `:a`    | ASCII                                                    |
| `:ms`   | timestamp in seconds (input in milliseconds)             |
| `:us`   | timestamp in seconds (input in microseconds)             |
| `:ts`   | timestamp in human-readable time (input in seconds)      |
| `:tms`  | timestamp in human-readable time (input in milliseconds) |
| `:tus`  | timestamp in human-readable time (input in microseconds) |
| `:cbor` | CBOR encoded items rendered in Diagnostic Notation (EDN) |

The first 4 display hints resemble what's supported in `core::fmt`, for example:

``` rust
# extern crate defmt;
defmt::info!("{=u8:x}", 42);  // -> INFO 2a
defmt::info!("{=u8:X}", 42);  // -> INFO 2A
defmt::info!("{=u8:#x}", 42); // -> INFO 0x2a
defmt::info!("{=u8:b}", 42);  // -> INFO 101010

defmt::info!("{=str}", "hello\tworld");   // -> INFO hello    world
defmt::info!("{=str:?}", "hello\tworld"); // -> INFO "hello\tworld"
```

The ASCII display hint formats byte slices (and arrays) using Rust's byte string syntax.

``` rust
# extern crate defmt;
let bytes = [104, 101, 255, 108, 108, 111];

defmt::info!("{=[u8]:a}", bytes); // -> INFO b"he\xffllo"
```

The CBOR display hint is useful when CBOR data is in memory, especially before further processing:

``` rust
# extern crate defmt;
# fn parse(slice: &[u8]) -> Result<(), ()> {
#     Ok(())
# }
# fn main() -> Result<(), ()> {
let id_cred = [0xa1, 0x04, 0x44, 0x6b, 0x69, 0x64, 0x31].as_slice();

defmt::info!("Peer ID: {=[u8]:cbor}", id_cred); // -> INFO Peer ID: {4: 'kid1'}
let parsed = parse(id_cred)?;
# Ok(())
# }
```

## Alternate printing

Adding `#` in front of a binary, octal, and hexadecimal display hints, precedes these numbers with a base indicator.

``` rust
# extern crate defmt;
defmt::info!("{=u8:b}", 42);  // -> INFO 101010
defmt::info!("{=u8:#b}", 42); // -> INFO 0b101010

defmt::info!("{=u8:o}", 42);  // -> INFO 52
defmt::info!("{=u8:#o}", 42); // -> INFO 0o52

defmt::info!("{=u8:x}", 42);  // -> INFO 2a
defmt::info!("{=u8:#x}", 42); // -> INFO 0x2a

defmt::info!("{=u8:X}", 42);  // -> INFO 2A
defmt::info!("{=u8:#X}", 42); // -> INFO 0x2A
```

## Zero padding

Padding numbers with leading zeros is supported, for example:

``` rust
# extern crate defmt;
defmt::info!("{=u8}", 42);    // -> INFO 42
defmt::info!("{=u8:04}", 42); // -> INFO 0042

defmt::info!("{=u8:08X}", 42);  // -> INFO 0000002A
defmt::info!("{=u8:#08X}", 42); // -> INFO 0x00002A
```

When the alternate form is used for hex, octal, and binary, the `0x`/`0o`/`0b` length is subtracted from the leading zeros.  This matches [`core::fmt` behavior](https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=b11809759f975e266251f7968e542756).

## Display hints for byte slice and byte array elements

Besides ASCII hints, byte slice and array elements can be formatted using hexadecimal, octal, or binary hints:

```rust
# extern crate defmt;
let bytes = [4, 101, 5, 108, 6, 111];

defmt::info!("{=[u8]}", bytes);         // -> INFO [4, 101, 5, 108, 6, 111]
defmt::info!("{=[u8]:x}", bytes);       // -> INFO [4, 65, 5, 6c, 6, 6f]
defmt::info!("{=[u8]:#04x}", bytes);    // -> INFO [0x04, 0x65, 0x05, 0x6c, 0x06, 0x6f]
defmt::info!("{=[u8]:#010b}", bytes);   // -> INFO [0b00000100, 0b01100101, 0b00000101, 0b01101100, 0b00000110, 0b01101111]
```

## Propagation

Display hints "propagate downwards" and apply to formatting parameters that specify no display hint.

``` rust
# extern crate defmt;
#[derive(defmt::Format)]
struct S { x: u8 }

let x = S { x: 42 };
defmt::info!("{}", x);    // -> INFO S { x: 42 }
defmt::info!("{:#x}", x); // -> INFO S { x: 0x2a }
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
defmt::info!("{}", x);   // -> INFO S { x: 42, y: 2a }
defmt::info!("{:b}", x); // -> INFO S { x: 101010, y: 2a }
```
