use log::{Log, Record};
use serde::Serialize;
use serde_json::{json, Value as JsonValue};

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
            let path = self::Path::new(record.module_path());
            let level = match record.is_println() {
                false => record.level().as_str(),
                true => "PRINTLN",
            };

            // defmt goes to stdout, since it's the primary output produced by this tool.
            let stdout = io::stdout();
            let mut sink = stdout.lock();

            serde_json::to_writer(
                &mut sink,
                &json!({
                    "backtrace": JsonValue::Null,
                    "data": record.args(),
                    "host_timestamp": chrono::Utc::now(),
                    "level": level,
                    "location": {
                        "file": record.file(),
                        "line": record.line(),
                    },
                    "path": path,
                    "target_timestamp": record.timestamp(),
                }),
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

#[derive(Serialize)]
struct Path {
    krate: String,
    modules: Vec<String>,
    function: String,
}

impl Path {
    fn new(module_path: Option<&str>) -> Option<Self> {
        let mut path = module_path?.split("::").collect::<Vec<_>>();

        // there need to be at least two elements, the crate and the function
        if path.len() < 2 {
            return None;
        };

        let function = path.pop()?.to_string();
        Some(Self {
            krate: path[..1][0].to_string(),
            modules: path[1..].iter().map(|a| a.to_string()).collect(),
            function,
        })
    }
}
