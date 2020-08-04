# Arrays

For arrays (`{:[u8; N]}`) the length is not serialized.

<!-- TODO remove ',ignore' -->
``` rust,ignore
# extern crate binfmt;
binfmt::error!("Data: {:[u8; 3]}!", [0, 1, 2]);
// on the wire: [1, 0, 1, 2]
//  string index ^  ^^^^^^^ the slice data
```
