# Logging macros

> Logging is done using the `error!`, `warn!`, `info!`, `debug!` and `trace!` macros.

Each macro logs at the logging level indicated by its name.
The syntax of these macros is roughly the same as the `println!`-macro.

``` rust
# extern crate defmt;
# let len = 80u8;
// -> INFO:  message arrived (length=80)
defmt::info!("message arrived (length={})", len);

# struct Message;
# impl Message { fn header(&self) -> u8 { 0 } }
# let message = Message;
// -> DEBUG: Header { source: 2, destination: 3, sequence: 16 }
defmt::debug!("{:?}", message.header());
```

## The `Format` trait

Unlike `core::fmt` which has several formatting traits (`Debug`, `Display`), `defmt` has a single formatting trait called `Format`.
The `{}` formatting parameter indicates that the `Format` trait will be used, meaning the argument must implement the `Format` trait.

``` rust
# extern crate defmt;
# let x = 0;
defmt::trace!("{}", x);
//                  ^ must implement the `Format` trait
```

## Type and display hints

The `defmt` grammer is similar to `core::fmt`, but not the same. It works like following:

> `{[pos][=Type][:Display]}`

### Type hint

The `Type` hint always starts with a `=`.
For once it enables the framework to further compress the logs resulting in higher throughput.
Secondly it also typechecks the supplied value to fit the specified type.

The type hint can be a [primitive](./primitives.md) or [one](./format-slices.md), [of](./istr.md), [the](./bitfields.md) special types.
Read on to learn more about the type hints.

### Display hint

The `Display` hint, always starts with a `:` and specifies the printing on the host side.
Read more about it [here](./hints.md).

### Positional parameter

The `pos` parameter lets you specify the position of the value to format (see ["Positional parameters"](https://doc.rust-lang.org/std/fmt/index.html#positional-parameters)).
