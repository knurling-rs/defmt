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

        let host_timestamp = Utc::now().timestamp_nanos();

        if let Some(record) = DefmtRecord::new(record) {
            // defmt goes to stdout, since it's the primary output produced by this tool.
            let stdout = io::stdout();
            let mut sink = stdout.lock();

            serde_json::to_writer(&mut sink, &JsonFrame::new(record, host_timestamp)).ok();
            writeln!(sink).ok();
        } else {
            // non-defmt logs go to stderr
            let stderr = io::stdout();
            let mut sink = stderr.lock();

            serde_json::to_writer(&mut sink, &JsonFrame::new_host(record, host_timestamp)).ok();
            writeln!(sink).ok();
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
    #[serde(skip_serializing_if = "Option::is_none")]
    is_host: Option<bool>,
    level: String,
    location: Option<Location>,
    target_timestamp: Option<String>,
}

impl JsonFrame {
    fn new(record: DefmtRecord, host_timestamp: i64) -> Self {
        let level = match record.is_println() {
            false => record.level().to_string(),
            true => "PRINTLN".to_string(),
        };

        Self {
            data: record.args().to_string(),
            decoder_version: env!("CARGO_PKG_VERSION"),
            host_timestamp,
            is_host: None,
            level,
            location: Location::new(record.file(), record.line(), record.module_path()),
            target_timestamp: Some(record.timestamp().to_string()),
        }
    }

    fn new_host(record: &Record, host_timestamp: i64) -> Self {
        Self {
            data: record.args().to_string(),
            decoder_version: env!("CARGO_PKG_VERSION"),
            host_timestamp,
            is_host: Some(true),
            level: record.level().to_string(),
            location: Location::new(record.file(), record.line(), record.module_path()),
            target_timestamp: None,
        }
    }

    pub fn data(&self) -> &str {
        self.data.as_str()
    }
    pub fn decoder_version(&self) -> &str {
        self.decoder_version
    }
    /// Unix timestamp in nanoseconds
    pub fn host_timestamp(&self) -> i64 {
        self.host_timestamp
    }
    /// Originates the log-frame from the host (`true`) or the target (`false`)?
    pub fn is_host(&self) -> bool {
        matches!(self.is_host, Some(true))
    }
    pub fn level(&self) -> &str {
        self.level.as_str()
    }
    pub fn target_timestamp(&self) -> Option<&String> {
        self.target_timestamp.as_ref()
    }

    // location attributes

    pub fn file(&self) -> Option<&String> {
        self.location.as_ref().map(|l| &l.file)
    }
    pub fn line(&self) -> Option<u32> {
        self.location.as_ref().map(|l| l.line)
    }
    pub fn crate_name(&self) -> Option<&String> {
        self.location.as_ref().map(|l| &l.crate_name)
    }
    pub fn modules(&self) -> Option<&Vec<String>> {
        self.location.as_ref().map(|l| &l.modules)
    }
    pub fn function(&self) -> Option<&String> {
        self.location.as_ref().map(|l| &l.function)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Location {
    file: String,
    line: u32,

    crate_name: String,
    modules: Vec<String>,
    function: String,
}

impl Location {
    fn new(file: Option<&str>, line: Option<u32>, module_path: Option<&str>) -> Option<Self> {
        let mut path = module_path?.split("::").collect::<Vec<_>>();

        // there need to be at least two elements, the crate and the function
        if path.len() < 2 {
            return None;
        };

        let function = path.pop()?.to_string();
        Some(Self {
            file: file?.to_string(),
            line: line?,

            crate_name: path[..1][0].to_string(),
            modules: path[1..].iter().map(|a| a.to_string()).collect(),
            function,
        })
    }
}
