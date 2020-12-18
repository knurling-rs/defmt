# Evaluation order

Consider this log invocation:

``` rust
# extern crate defmt;
defmt::info!("x={=?}", foo());

fn foo() -> u8 {
    defmt::info!("Hello");
    42
}
```

Depending on *when* `foo` is invoked this can result in potential re-entrancy / nesting and cause `info!("Hello")` to be lost.
So we'll make the macro evaluate format arguments *before* the acquire operation.
Something like this:
(`core::fmt` does a similar `match` operation)

``` rust
# struct Logger;
# impl Logger {
#     fn acquire() -> Option<Self> { None }
# }
# fn foo() -> u8 { 0 }
match (foo()) { // evaluate formatting arguments
    (_0) => {
        if let Some(logger) = Logger::acquire() {
            // serialize `_0`, etc.
        }
    }
}
```
