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
            let path = self::Path::new(record.file(), record.line(), record.module_path());
            let level = match record.is_println() {
                false => record.level().as_str(),
                true => "PRINTLN",
            }
            .to_string();

            // defmt goes to stdout, since it's the primary output produced by this tool.
            let stdout = io::stdout();
            let mut sink = stdout.lock();

            serde_json::to_writer(
                &mut sink,
                &Json {
                    backtrace: None,
                    data: record.args().to_string(),
                    host_timestamp: Utc::now().timestamp_nanos(),
                    level,
                    path,
                    target_timestamp: record.timestamp().to_string(),
                },
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
pub struct Json {
    data: String,
    /// Unix timestamp in nanoseconds
    host_timestamp: i64,
    level: String,
    path: Option<Path>,
    target_timestamp: String,

    // backtrace is omitted from output if it is `None`
    #[serde(skip_serializing_if = "Option::is_none")]
    backtrace: Option<Vec<()>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Path {
    file: String,
    line: u32,

    krate: String,
    modules: Vec<String>,
    function: String,
}

impl Path {
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

            krate: path[..1][0].to_string(),
            modules: path[1..].iter().map(|a| a.to_string()).collect(),
            function,
        })
    }
}
