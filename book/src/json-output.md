# JSON output

> Structured logging for `probe-run`.

As an alternative to its human-focused output, `probe-run` offers structured JSON output. There are two use-cases:
- building software on top of `probe-run`
- storing `probe-run`'s output, in order to analyze it over time

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

There really are no boundaries. You can process the JSON with any programming language you like. If you want to do that in rust, we provide some data transfer objects which implement `serde`'s `trait Serialize` and `trait Deserialize`, in the [`defmt-json-schema`] crate.

See how they can be use in [this example](https://github.com/knurling-rs/defmt/blob/b396257be01f477eda2ac16f7e62dece31749963/decoder/defmt-json-schema/examples/simple.rs) and how they are defined below.

### JsonFrame

The most important one is the `struct JsonFrame`. It contains all the information for one log-statement.

It is defined like this:

```rust
pub struct JsonFrame {
    pub data: String,
    pub host_timestamp: i64,
    pub level: Option<log::Level>,
    pub location: Location,
    pub target_timestamp: String,
}

pub struct Location {
    pub file: Option<String>,
    pub line: Option<u32>,
    pub module_path: Option<ModulePath>,
}

pub struct ModulePath {
    pub crate_name: String,
    pub modules: Vec<String>,
    pub function: String,
}

# mod log { pub struct Level; }
```

... which results in following JSON schema ...

```json
{
    "data": "string",
    "host_timestamp": "number",
    "level": null | "string",
    "location": {
        "file": null | "string",
        "line": null | "number",
        "module_path": null | {
            "crate_name": "string",
            "modules": ["string"],
            "function": "string"
        }
    },
    "target_timestamp": "string"
}
```

... which looks like this, for example:

```json
{
    "data": "Hello, world!",
    "host_timestamp": 1643110723881550007,
    "level": "INFO",
    "location": {
        "file": "src/bin/hello.rs",
        "line": 8,
        "module_path": {
            "crate_name": "hello",
            "modules": [],
            "function": "__cortex_m_rt_main"
        }
    },
    "target_timestamp": "1"
}
```

### SchemaVersion

The second type provided, is the `struct SchemaVersion`:

```rust
pub struct SchemaVersion {
    pub schema_version: u32,
}
```

... which results in following JSON schema ...

```json
{
    "schema_version": "number"
}
```

... which looks like this, for example:

```json
{
    "schema_version": 1
}
```

[`defmt-json-schema`]: https://crates.io/crates/defmt-json-frame