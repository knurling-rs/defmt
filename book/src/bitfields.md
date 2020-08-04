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
