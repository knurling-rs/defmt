# write!

When implementing the `Format` trait manually, the `write!` macro must be used to log the data.
This macro takes a `Formatter` as its first argument.

``` rust
/// Packet configuration register 1
pub struct PCNF1 { value: u32 }

impl binfmt::Format for PCNF1 {
    fn fmt(&self, f: &mut binfmt::Formatter) {
        binfmt::write!(
            f,
            "PCNF1: {{ MAXLEN: {0:0..8}, STATLEN: {0:8..16}, BALEN: {0:16..19} }}",
            self.value,
        );
    }
}
```
