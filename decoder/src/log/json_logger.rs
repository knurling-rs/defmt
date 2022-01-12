use log::{Log, Record};
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
            let (krate, modules, function, is_method) = extract_path(record.module_path());
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
                        "column": "TODO",
                    },
                    "path": {
                        "crate": krate,
                        "modules": modules,
                        "function": function,
                        "is_method": is_method,
                    },
                    "target_timestamp": record.timestamp(),
                }),
            )
            .ok();
            writeln!(sink).ok();
        } else { /* TODO: handle host logs */
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

fn extract_path(
    module_path: Option<&str>,
) -> (
    Option<String>,
    Option<Vec<String>>,
    Option<String>,
    Option<bool>,
) {
    let module_path = if let Some(a) = module_path {
        a.split("::").collect::<Vec<_>>()
    } else {
        return (None, None, None, None);
    };

    let idx = module_path.len() - 1;

    let krate = module_path[..1][0].to_string();
    let modules = module_path[1..idx]
        .iter()
        .map(|b| b.to_string())
        .collect::<Vec<_>>();
    let function = module_path[idx..][0].to_string();

    let is_method = modules.iter().any(|m| m.starts_with("{impl#"));

    (Some(krate), Some(modules), Some(function), Some(is_method))
}
