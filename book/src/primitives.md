# Primitives

In addition to `{}` there are formatting parameters for several primitive types.
These parameters follow the syntax `{=Type}`, for example: `{=u8}`, `{=bool}`.
This type information lets the framework further compress the logs resulting in higher throughput.

``` rust
# extern crate defmt;
# let enabled = false;
# let ready = false;
# let timeout = false;
// arguments can be compressed into a single byte
defmt::info!(
    "enabled: {=bool}, ready: {=bool}, timeout: {=bool}",
    enabled, ready, timeout,
);

# let x = 0u16;
// arguments will be type checked
defmt::trace!("{=u16}", x);
//                      ^ must have type `u16`
```

The available types are:

- `=bool`, boolean
- `={i,u}{8,16,32,64}`, standard integer types
- `={i,u}24`, 32-bit integer truncated to 24 bits
- `=f32`, 32-bit floating point type
- `=f64`, 64-bit floating point type
- `=[u8; N]`, byte array
- `=[u8]`, byte slice
- `=str`, string slice
