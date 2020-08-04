# Format

The untyped argument (`:?`) requires one level of indirection during serialization.

First let's see how a primitive implements the `Format` trait:

``` rust
impl Format for u8 {
    fn format(&self, f: &mut Formatter) {
        binfmt::write!(f, "{:u8}", self)
        // on the wire: [1, 42]
        //  string index ^  ^^ `self`
        //  ^ = intern("{:u8}")
    }
}
```

`Format` will use the `write!` macro.
This will send the string index of `{:u8}` followed by the one-byte data.
In general, `write!` can use `{:?}` so `Format` nesting is possible.

Now let's look into a log invocation:

``` rust
binfmt::error!("The answer is {:?}!", 42u8);
// on the wire: [2, 1, 42]
//  string index ^  ^^^^^ `42u8.format(/*..*/)`
//  ^ = intern("The answer is {:?}!")
```

This will send the string index of "The answer is {:?}!" and invoke the argument's `Format::format` method.

> TODO(japaric) a naive `[T]`'s `Format` implementation (`slice.for_each(format)`) has high overhead: the string index of e.g. `{:u8}` would be repeated N times.
> We'll need to some specialization to avoid that repetition.
