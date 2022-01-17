use chrono::Utc;
use log::{Log, Record};
use serde::{Deserialize, Serialize};

use std::io::{self, Write};

use super::DefmtRecord;

pub(crate) struct JsonLogger {
    should_log: Box<dyn Fn(&log::Metadata) -> bool + Sync + Send>,
}

impl Log for JsonLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
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

            serde_json::to_writer(
                &mut sink,
                &JsonFrame::new(record, Utc::now().timestamp_nanos()),
            )
            .ok();
            writeln!(sink).ok();
        } else {
            // TODO: handle host logs
        }
    }

    fn flush(&self) {}
}

impl JsonLogger {
    pub fn new(should_log: impl Fn(&log::Metadata) -> bool + Sync + Send + 'static) -> Box<Self> {
        Box::new(Self {
            should_log: Box::new(should_log),
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonFrame {
    data: String,
    decoder_version: &'static str,
    host_timestamp: i64,
    level: String,
    location: Option<Location>,
    target_timestamp: String,
}

impl JsonFrame {
    fn new(record: DefmtRecord, host_timestamp: i64) -> Self {
        let location = self::Location::new(record.file(), record.line(), record.module_path());
        let level = match record.is_println() {
            false => record.level().as_str(),
            true => "PRINTLN",
        }
        .to_string();

        Self {
            data: record.args().to_string(),
            decoder_version: env!("CARGO_PKG_VERSION"),
            host_timestamp,
            level,
            location,
            target_timestamp: record.timestamp().to_string(),
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
    pub fn level(&self) -> &str {
        self.level.as_str()
    }
    pub fn target_timestamp(&self) -> &str {
        self.target_timestamp.as_str()
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
