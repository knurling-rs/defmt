# Format Slices

The `{:[:?]}` parameter can be used to log a slices of values that implement the `Format` trait.
The expected argument is a slice.

``` rust
struct X {
    y: u16,
    z: u8,
}
let xs: &[X] = &[/* .. */];
info!("xs={:[:?]}")
```

Note that for byte of slices `{:[u8]}` should be preferred as it's better compressed.
`[T] where T: Format` also implements the `Format` trait so it's possible to format `[T]` with `{:?}` but `{:[?]}` uses slightly less bandwidth.
