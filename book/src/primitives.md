# Primitives

In addition to `:?` there are formatting parameters for several primitive types.
These parameters follow the syntax `:<type>`, for example: `:u8`, `:bool`.
This type information lets the framework further compress the logs resulting in higher throughput.

``` rust
// arguments can be compressed into a single byte
binfmt::info!(
    "enabled: {:bool}, ready: {:bool}, timeout: {:bool}",
    enabled, ready, timeout,
);

// arguments will be type checked
binfmt::trace!("{:u16}", x);
//                       ^ must have type `u16`
```

The available types are:

- `:bool`, boolean
- `:{i,u}{8,16,32}`, standard integer types
- `:{i,u}24`, 32-bit integer truncated to 24 bits
- `:f32`, 32-bit floating point type
- `:[u8; N]`, byte array
- `:[u8]`, byte slice
- `:str`, string slice
