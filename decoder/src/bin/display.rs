use decoder::Table;
use std::collections::BTreeMap;

fn main() {
    let mut entries = BTreeMap::new();
    entries.insert(0, "Hello, world!".to_owned());
    entries.insert(1, "The answer is {:u8}!".to_owned());
    // [IDX, TS, 42]
    //           ^^
    //entries.insert(2, "The answer is {0:u8} {1:u16}!".to_owned());

    let table = Table {
        entries,
        debug: 1..2,
        error: 0..0,
        info: 0..1,
        trace: 0..0,
        warn: 0..0,
    };

    let bytes = [0, 1];
    //     index ^  ^ timestamp
    let frame = decoder::decode(&bytes, &table).unwrap();
    println!("{}", frame.0);
}
