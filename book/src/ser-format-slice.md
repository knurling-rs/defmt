# Format Slices

`{:[?]}` will serialize the length (LEB128 compressed) first, then the first element will be serialized in (recursively) *tagged* format. The rest of elements will be serialized *untagged*.

"Tagged" means that the data will be preceded by the string indices that indicate how to format the data.

Example:

``` rust
# extern crate binfmt;
use binfmt::Format;

#[derive(Format)]
struct X {
    y: Y,
}

#[derive(Format)]
struct Y {
    z: u8,
}

fn serialize() {
    let xs = [X { y: Y { z: 42 }}, X { y: Y { z: 24 }}];
    binfmt::info!("{:[?]}", &xs[..]);
    // on-the-wire: [
    //     1,  // "{:[?]}"
    //     2,  // `leb(xs.len())`
    //     2,  // "X {{ y: {:?} }}"  / outer tag
    //     3,  // "Y {{ z: {:u8} }}" / inner tag
    //     42, // xs[0].y.z
    //     (no tags for the second element)
    //     24, // xs[1].y.z
    // ]
}
```
