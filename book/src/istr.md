# Interned strings

The `{:istr}` formatting parameter is used for *interned* strings.
Compared to the `{:str}` parameter, which transmits a complete string, `{:istr}` saves bandwidth by sending only a string index.
The `{:istr}` parameter expects an argument with type `binfmt::Str`.
A `Str` value is created using the `intern!` macro; the argument to this macro must be a string literal.

``` rust
let s = "The quick brown fox jumps over the lazy dog";
info!("{:str}", s);
//              ^ bandwidth-use = 43 bytes

let interned: Str = intern!("The quick brown fox jumps over the lazy dog");
info!("{:istr}", interned);
//               ^^^^^^^^ bandwidth-use <= 2 bytes
```

This was a contrived example to show the difference in bandwidth use.
In practice you should use:

``` rust
info!("The quick brown fox jumps over the lazy dog");
```

which also interns the log string and uses as little bandwidth as the `{:istr}` version.
