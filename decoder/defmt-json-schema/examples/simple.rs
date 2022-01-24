use std::fs;

use defmt_json_schema::{v1, SchemaVersion};

fn main() {
    let s = fs::read_to_string("examples/simple.json").unwrap();
    let lines = s.lines().collect::<Vec<_>>();

    let schema_version: SchemaVersion = serde_json::from_str(lines[0]).unwrap();

    if schema_version == v1::SCHEMA_VERSION {
        println!("Detected version \"1\" of JsonFrame!");
        for &data in lines[1..].iter() {
            let json_frame: v1::JsonFrame = serde_json::from_str(data).unwrap();
            println!("Msg: {:?}", json_frame);
        }
    };
}
