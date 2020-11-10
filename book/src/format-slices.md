# Format slices / arrays

The `{:[?]}` parameter can be used to log a slices of values that implement the `Format` trait.
The expected argument is a slice.

``` rust
# extern crate defmt;
# use defmt::{Format, info};
#[derive(Format)]
struct X {
    y: u16,
    z: u8,
}
let xs: &[X] = &[/* .. */];
info!("xs={:[?]}", xs);
```

Note that for slices of bytes `{:[u8]}` should be preferred as it's better compressed.
`[T] where T: Format` also implements the `Format` trait so it's possible to format `[T]` with `{:?}` but `{:[?]}` uses slightly less bandwidth.

If you have an array of types that implement the `Format` trait, instead of a slice. You should use
the `{:[?; N]}` parameter (where `N` is a number); this saves bandwidth compared to `{:[?]}`.

``` rust
# extern crate defmt;
# use defmt::{Format, info};
#[derive(Format)]
struct X {
    y: u16,
    z: u8,
}
let xs: [X; 2] = [
# X { y: 1, z: 2 },
# X { y: 3, z: 4 },
    // ..
];
info!("xs={:[?; 2]}", xs);
```
