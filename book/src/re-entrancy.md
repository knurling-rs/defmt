# Re-entrancy

> [!IMPORTANT]
> The design and implementation chapter is outdated

Where can re-entrancy occur?
Turns out that with global singletons it can occur about anywhere; you don't need interrupts (preemption) to cause re-entrancy.
See below:

``` rust
# extern crate defmt;
# let x = 0u8;
defmt::info!("The answer is {=?}!", x /*: Struct */);
```

As you have seen before this will first send the string index of "The answer is {=?}!" and then call `x`'s `Format::format` method.
The re-entrancy issue arises if the `Format` implementation calls a logging macro:

``` rust
# extern crate defmt;
# struct X;
impl defmt::Format for X {
    fn format(&self, f: defmt::Formatter) {
        //           ^ this is a handle to the global logger
        defmt::info!("Hello!");
        // ..
    }
}
```

`f` is a handle to the global logger.
The `info!` call inside the `format` method is trying to access the global logger again.
If `info!` succeeds then you have two exclusive handles (`Formatter`) to the logger and that's UB.
If `info!` uses a spinlock to access the logger then this will deadlock.
