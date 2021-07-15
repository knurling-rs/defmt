# Primitives

The following **primitive types** are available:

| type hint            | name                                |
| :------------------- | :---------------------------------- |
| `=bool`              | boolean                             |
| `={i,u}{8,16,32,64}` | standard integer types              |
| `=f{32, 64}`         | 32-bit / 64-bit floating point type |
| `=[u8; N]`           | byte array                          |
| `=[u8]`              | byte slice                          |
| `=str`               | string slice                        |

They can be used like this:

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

---

Additionally there are some **special types**:

| type hint | name             |
| :-------- | :--------------- |
| `=M..N`   | Bitfields        |
| `=istr`   | Interned Strings |
| `=[?]`    | Format slices    |
| `=[?; N]` | Format arrays    |

Read more about them in the following chapters.
