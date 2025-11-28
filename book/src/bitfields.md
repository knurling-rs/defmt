# Bitfields

> `:M..N` is the bitfield formatting parameter.

The bitfield argument is expected to be a *unsigned* integer that's large enough to contain the bitfields.
For example, if bitfield ranges only cover up to bit `11` (e.g. `=8..12`) then the argument must be at least `u16`.

When paired with a positional parameter it can be used to display the bitfields of a register.
``` rust
# extern crate defmt;
# let pcnf1 = 0u32;
// -> TRACE: PCNF1 { MAXLEN: 125, STATLEN: 3, BALEN: 2 }
defmt::trace!(
    "PCNF1: {{ MAXLEN: {0=0..8}, STATLEN: {0=8..16}, BALEN: {0=16..19} }}",
    //                  ^                  ^                 ^ same argument
    pcnf1, // <- type must be `u32`
);
```

Bitfields are not range inclusive, e.g. following statement will evaluate to `5` (`0b110`):
``` rust
# extern crate defmt;
// -> TRACE: first three bits: 110
defmt::trace!("first three bits: {0=0..3}", 254u32);
```

> [!IMPORTANT]
> You can not reuse the same argument in a bitfield- and a non bitfield parameter.
> 
> This will not compile:
> ``` rust,compile_fail
> # extern crate defmt;
> defmt::trace!("{0=5..13} {0=u16}", 256u16);
> ```
