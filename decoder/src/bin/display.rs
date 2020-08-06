use decoder::Table;
use std::collections::BTreeMap;

fn main() {
    let mut entries = BTreeMap::new();
    entries.insert(0, "x={:?}".to_owned());
    entries.insert(1, "Foo {{ x: {:f32} }}".to_owned());

    let table = Table {
        entries,
        debug: 0..0,
        error: 0..0,
        info: 0..1,
        trace: 0..0,
        warn: 0..0,
    };

    let mut bytes = vec![
        0, // index
        2, // timestamp
        1, // index of the struct
    ];
    bytes.extend_from_slice(&f32::to_bits(1.1e-10).to_le_bytes());

    let frame = decoder::decode(&bytes, &table).unwrap();
    println!("{}", frame.0.display(true));
    println!("{}", 1.1e-10);
}
