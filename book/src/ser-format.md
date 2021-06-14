# Format

The untyped argument (`=?`) requires one level of indirection during serialization.

First let's see how a primitive implements the `Format` trait:

``` rust
# extern crate defmt;
# macro_rules! internp { ($l:literal) => { 0 } }
# trait Format { fn format(&self, fmt: defmt::Formatter); }
impl Format for u8 {
    fn format(&self, fmt: defmt::Formatter) {
        let t = internp!("{=u8}");
        fmt.inner.tag(&t);
        fmt.inner.u8(self)
        // on the wire: [1, 42]
        //  string index ^  ^^ `self`
    }
}
```

`Format` will use the `write!` macro.
This will send the string index of `{=u8}` followed by the one-byte data.
In general, `write!` can use `{=?}` so `Format` nesting is possible.

Now let's look into a log invocation:

``` rust
# extern crate defmt;
defmt::error!("The answer is {=?}!", 42u8);
// on the wire: [2, 1, 42]
//  string index ^  ^^^^^ `42u8.format(/*..*/)`
//  ^ = intern("The answer is {=?}!")
```

This will send the string index of "The answer is {:?}!" and invoke the argument's `Format::format` method.
