use chrono::Utc;
use log::{Log, Metadata, Record};
use serde::{Deserialize, Serialize};

use std::io::{self, Write};

use super::DefmtRecord;

pub(crate) struct JsonLogger {
    should_log: Box<dyn Fn(&Metadata) -> bool + Sync + Send>,
}

impl Log for JsonLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        (self.should_log)(metadata)
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        if let Some(record) = DefmtRecord::new(record) {
            // defmt goes to stdout, since it's the primary output produced by this tool.
            let stdout = io::stdout();
            let mut sink = stdout.lock();

            let host_timestamp = Utc::now().timestamp_nanos();
            serde_json::to_writer(&mut sink, &JsonFrame::new(record, host_timestamp)).ok();
            writeln!(sink).ok();
        } else {
            // non-defmt logs are dropped
        }
    }

    fn flush(&self) {}
}

impl JsonLogger {
    pub fn new(should_log: impl Fn(&Metadata) -> bool + Sync + Send + 'static) -> Box<Self> {
        Box::new(Self {
            should_log: Box::new(should_log),
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonFrame {
    // please find the attribute documentation at the getter-methods
    //
    data: String,
    decoder_version: &'static str,
    host_timestamp: i64,
    level: String,
    location: Location,
    target_timestamp: String,
}

impl JsonFrame {
    /// Create a new [JsonFrame] from a log-frame from the target
    fn new(record: DefmtRecord, host_timestamp: i64) -> Self {
        let level = match record.is_println() {
            false => record.level().to_string(),
            true => "PRINTLN".to_string(),
        };

        Self {
            data: record.args().to_string(),
            decoder_version: env!("CARGO_PKG_VERSION"),
            host_timestamp,
            level,
            location: Location {
                file: record.file().map(|f| f.to_string()),
                line: record.line(),
                module_path: ModulePath::new(record.module_path()),
            },
            target_timestamp: record.timestamp().to_string(),
        }
    }

    pub fn data(&self) -> &str {
        self.data.as_str()
    }
    /// `defmt-decoder`-version the log-frame was produced with
    pub fn decoder_version(&self) -> &str {
        self.decoder_version
    }
    /// Unix timestamp in nanoseconds
    pub fn host_timestamp(&self) -> i64 {
        self.host_timestamp
    }
    pub fn level(&self) -> &str {
        self.level.as_str()
    }
    pub fn target_timestamp(&self) -> &str {
        self.target_timestamp.as_str()
    }

    // location attributes

    pub fn file(&self) -> &Option<String> {
        &self.location.file
    }
    pub fn line(&self) -> &Option<u32> {
        &self.location.line
    }
    pub fn crate_name(&self) -> Option<&String> {
        self.location.module_path.as_ref().map(|l| &l.crate_name)
    }
    pub fn modules(&self) -> Option<&Vec<String>> {
        self.location.module_path.as_ref().map(|l| &l.modules)
    }
    pub fn function(&self) -> Option<&String> {
        self.location.module_path.as_ref().map(|l| &l.function)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Location {
    file: Option<String>,
    line: Option<u32>,
    module_path: Option<ModulePath>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ModulePath {
    crate_name: String,
    modules: Vec<String>,
    function: String,
}

impl ModulePath {
    fn new(module_path: Option<&str>) -> Option<Self> {
        let mut path = module_path?.split("::").collect::<Vec<_>>();

        // there need to be at least two elements, the crate and the function
        if path.len() < 2 {
            return None;
        };

        // the last element is the function
        let function = path.pop()?.to_string();
        // the first element is the crate_name
        let crate_name = path.remove(0).to_string();

        Some(Self {
            crate_name,
            modules: path.into_iter().map(|a| a.to_string()).collect(),
            function,
        })
    }
}
