# JSON output

> Structured logging for `probe-run`.

As an alternative to its human-focused output, `probe-run` offers structured JSON output. There are two use-cases:
- building software on top of `probe-run`
- storing `probe-run`'s output, in order to analyze it over time

⚠️ `probe-run` v0.3.3+ is necessary to use this feature!


## How to use it?

> 😁: Sounds great, how can I use it?

To activate the JSON output, just add the `--json` flag to your invocation of `probe-run`. If you are using our `app-template` edit `.cargo/config.toml` like this:

```diff
[target.'cfg(all(target_arch = "arm", target_os = "none"))']
- runner = "probe-run --chip $CHIP"
+ runner = "probe-run --chip $CHIP --json"
```

Now `probe-run` will output one line with a JSON object for each log-statement to `stdout`, and your output will look similar to this:

```console
$ DEFMT_LOG=debug cargo run --bin levels

{"schema_version":1}
(HOST) INFO  flashing program (2 pages / 8.00 KiB)
└─ probe_run @ src/main.rs:93
(HOST) INFO  success!
└─ probe_run @ src/main.rs:126
────────────────────────────────────────────────────────────────────────────────
{"data":"info","host_timestamp":1643113115873940726,"level":"INFO","location":{"file":"src/bin/levels.rs","line":10,"module_path":{"crate_name":"levels","modules":[],"function":"__cortex_m_rt_main"}},"target_timestamp":"0"}
{"data":"warn","host_timestamp":1643113115873952269,"level":"WARN","location":{"file":"src/bin/levels.rs","line":12,"module_path":{"crate_name":"levels","modules":[],"function":"__cortex_m_rt_main"}},"target_timestamp":"1"}
{"data":"debug","host_timestamp":1643113115873957827,"level":"DEBUG","location":{"file":"src/bin/levels.rs","line":13,"module_path":{"crate_name":"levels","modules":[],"function":"__cortex_m_rt_main"}},"target_timestamp":"2"}
{"data":"error","host_timestamp":1643113115873981443,"level":"ERROR","location":{"file":"src/bin/levels.rs","line":14,"module_path":{"crate_name":"levels","modules":[],"function":"__cortex_m_rt_main"}},"target_timestamp":"3"}
{"data":"println","host_timestamp":1643113115873987212,"level":null,"location":{"file":"src/bin/levels.rs","line":15,"module_path":{"crate_name":"levels","modules":[],"function":"__cortex_m_rt_main"}},"target_timestamp":"4"}
────────────────────────────────────────────────────────────────────────────────
(HOST) INFO  device halted without error
└─ probe_run::backtrace @ src/backtrace/mod.rs:108
```

> 🤯: But wait a moment?! ... That is not only JSON! How am I supposed to process that?

That is easy. As mentioned, the JSON output goes to `stdout`. All the other output, like host logs and backtraces go to `stderr` and therefore can be processed separately.

For example, you can redirect the JSON output to a file and still see the host logs in the terminal:

```console
$ DEFMT_LOG=debug cargo rb levels > levels.json

(HOST) INFO  flashing program (2 pages / 8.00 KiB)
└─ probe_run @ src/main.rs:93
(HOST) INFO  success!
└─ probe_run @ src/main.rs:126
────────────────────────────────────────────────────────────────────────────────
────────────────────────────────────────────────────────────────────────────────
(HOST) INFO  device halted without error
└─ probe_run::backtrace @ src/backtrace/mod.rs:108
```

Afterwards `levels.json` looks like this:
```json
{"schema_version":1}
{"data":"info","host_timestamp":1643113389707243978,"level":"INFO","location":{"file":"src/bin/levels.rs","line":10,"module_path":{"crate_name":"levels","modules":[],"function":"__cortex_m_rt_main"}},"target_timestamp":"0"}
{"data":"warn","host_timestamp":1643113389707290115,"level":"WARN","location":{"file":"src/bin/levels.rs","line":12,"module_path":{"crate_name":"levels","modules":[],"function":"__cortex_m_rt_main"}},"target_timestamp":"1"}
{"data":"debug","host_timestamp":1643113389707299759,"level":"DEBUG","location":{"file":"src/bin/levels.rs","line":13,"module_path":{"crate_name":"levels","modules":[],"function":"__cortex_m_rt_main"}},"target_timestamp":"2"}
{"data":"error","host_timestamp":1643113389707306961,"level":"ERROR","location":{"file":"src/bin/levels.rs","line":14,"module_path":{"crate_name":"levels","modules":[],"function":"__cortex_m_rt_main"}},"target_timestamp":"3"}
{"data":"println","host_timestamp":1643113389707313290,"level":null,"location":{"file":"src/bin/levels.rs","line":15,"module_path":{"crate_name":"levels","modules":[],"function":"__cortex_m_rt_main"}},"target_timestamp":"4"}
```
> 🤔: That seems convenient, but what is this schema version in the first line?

It indicates the version of the json format you are using. `probe-run` will always output it as a header at the beginning of each stream of logs. We anticipate that the format will slightly change while `probe-run` and `defmt` evolve. Using this version you always know which revision is in use and can act upon that.

> 🤗: Sounds great!
## Data transfer objects

> 🤔: So, what can I do with the JSON output?

There really are no boundaries. You can process the JSON with any programming language you like and also store it any data store of your choice to process and analyze it later. If you are saving the output for later, it might make sense to store the schema version together with additional metadata like e.g. a device id or firmware version. One option is to use a program like jq to extract the parts of interest.

If you wish to deserialize the entire data back into a Rust program, you will need to be able to decode the `SchemaVersion` object at the start of the stream, as well as the `JsonFrame` objects which follow after the schema version. To do that, we supply a few things in [`defmt_json_schema`]:
  - a `SchemaVersion` struct in `defmt_json_schema::SchemaVersion`,
  - a versioned `JsonFrame` struct in `defmt_json_schema::{schema_version}::JsonFrame` and
  - a `SCHEMA_VERSION` constant for each version of the `JsonFrame` in `defmt_json_schema::{version}::SCHEMA_VERSION`.
 
[`defmt_json_schema`]: https://crates.io/crates/defmt-json-schema

You can use all of this together with `serde_json` like following:

``` rust
# extern crate defmt_json_schema;
# extern crate serde_json;

use defmt_json_schema::{v1, SchemaVersion};

const DATA: &str = r#"{"schema_version":1}
{"data":"Hello, world!","host_timestamp":1642698490360848721,"level":null,"location":{"file":"src/bin/hello.rs","line":9,"module_path":{"crate_name":"hello","modules":[],"function":"__cortex_m_rt_main"}},"target_timestamp":"0"}
{"data":"S { a: 8 }","host_timestamp":1642698490361019228,"level":"INFO","location":{"file":"src/bin/hello.rs","line":26,"module_path":{"crate_name":"hello","modules":["{impl#0}"],"function":"abc"}},"target_timestamp":"1"}"#;

fn main() {
    let mut data = DATA.lines().collect::<Vec<_>>();

    // first we decode the schema version
    let schema_version: SchemaVersion = serde_json::from_str(data[0]).unwrap();

    // and then handle the rest of the data (depending on the schema version)
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
```

You can find an example with reading the content from a file [here](https://github.com/knurling-rs/defmt/blob/main/decoder/defmt-json-schema/examples/simple.rs).

[`defmt-json-schema`]: https://crates.io/crates/defmt-json-frame