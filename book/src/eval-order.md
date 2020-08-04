# Evaluation order

Consider this log invocation:

``` rust
binfmt::info!("x={:?}", foo());

fn foo() {
    binfmt::info!("Hello");
}
```

Depending on *when* `foo` is invoked this can result in potential re-entrancy / nesting and cause `info!("Hello")` to be lost.
So we'll make the macro evaluate format arguments *before* the acquire operation.
Something like this:
(`core::fmt` does a similar `match` operation)

``` rust
match (foo()) { // evaluate formatting arguments
    (_0) => {
        if let Some(logger) = Logger::acquire() {
            // serialize `_0`, etc.
        }
    }
}
```
