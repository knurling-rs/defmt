use std::fs;

use defmt_json_schema::{v1, SchemaVersion};

fn main() {
    let s = fs::read_to_string("examples/simple.json").unwrap();
    let data = s.lines().collect::<Vec<_>>();

    let schema_version: SchemaVersion = serde_json::from_str(data[0]).unwrap();

    match schema_version {
        v1::SCHEMA_VERSION => handle_v1(&data[1..]),
        // v2::SCHEMA_VERSION => handle_v2(&data[1..]),
        _ => unreachable!(),
    };
}

fn handle_v1(data: &[&str]) {
    println!("Detected version \"1\" of JsonFrame!");
    use v1::JsonFrame;

    for &data in data.iter() {
        let json_frame: JsonFrame = serde_json::from_str(data).unwrap();
        println!("{:?}", json_frame);
    }
}
