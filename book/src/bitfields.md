# Bitfields

`:m..n` is the bitfield formatting parameter.
When paired with a positional parameter it can be used to display the bitfields of a register.

``` rust
// -> TRACE: PCNF1 { MAXLEN: 125, STATLEN: 3, BALEN: 0b010 }
binfmt::trace!(
    "PCNF1: {{ MAXLEN: {0:0..8}, STATLEN: {0:8..16}, BALEN: {0:16..19} }}",
    //                  ^                  ^                 ^ same argument
    pcnf1, // <- type must be `u32`
);
```

The bitfield argument is expected to be a *unsigned* integer that's just large enough to contain the bitfields.
For example, if bitfield ranges only cover up to bit `11` (e.g. `:8..12`) then the argument must be `u16`.

Bit indices are little-endian: the 0th bit is the rightmost bit.

Bitfields are not range inclusive, e.g.
``` rust
binfmt::trace!("first two bits: {0:0..3}", 254);
```
will evaluate to `0b10`.


⚠️ Currently, the bitfielded argument must be of the smallest type that can contain the largest end index.

``` rust
// this will not compile
binfmt::trace!("{0:0..3}", 555u16);

// this will compile
binfmt::trace!("{0:0..3}", 555u16 as u8);
```

⚠️ You can not reuse the same argument in a bitfield- and a non bitfield parameter. This will not compile:
``` rust
binfmt::trace!("{0:0..3} {0:u16}", 256u16);
```