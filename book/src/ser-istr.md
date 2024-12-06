# Interned strings

> ⚠️ The design and implementation chapter is outdated ⚠️

Let's ignore timestamps for now and also ignore how access to the global logger is synchronized.
This is the simplest case: logging a string literal with no formatting.

``` rust
# extern crate defmt;
defmt::info!("Hello, world!");
```

As we saw in the previous section this string will get interned.
Interning converts the string into a `usize` index.
This `usize` index will be further compressed using [LEB128].
Some examples: (values on the right are `u8` arrays)

[LEB128]: https://en.wikipedia.org/wiki/LEB128

- `1usize` -> `[1]`
- `127usize` -> `[127]`
- `128usize` -> `[128, 1]`
- `255usize` -> `[255, 1]`

Because string indices start at zero and it's unlikely that a program will intern more than 2^14 strings string literals will be serialized as 1 or 2 bytes indices.
