use chrono::Utc;
use defmt_json_schema::v1::{JsonFrame, Location, ModulePath};
use log::{Log, Metadata, Record};

use std::io::{self, Write};

use super::{pretty_logger::PrettyLogger, DefmtRecord};

pub(crate) struct JsonLogger {
    should_log: Box<dyn Fn(&Metadata) -> bool + Sync + Send>,
    host_logger: PrettyLogger,
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
            serde_json::to_writer(&mut sink, &create_json_frame(record, host_timestamp)).ok();
            writeln!(sink).ok();
        } else {
            // non-defmt logs go to stderr
            let stderr = io::stderr();
            let sink = stderr.lock();

            self.host_logger.print_host_record(record, sink);
        }
    }

    fn flush(&self) {}
}

impl JsonLogger {
    pub fn new(should_log: impl Fn(&Metadata) -> bool + Sync + Send + 'static) -> Box<Self> {
        Box::new(Self {
            should_log: Box::new(should_log),
            host_logger: PrettyLogger::new_unboxed(true, |_| true),
        })
    }
}

/// Create a new [JsonFrame] from a log-frame from the target
fn create_json_frame(record: DefmtRecord, host_timestamp: i64) -> JsonFrame {
    let level = match record.is_println() {
        false => record.level().to_string(),
        true => "PRINTLN".to_string(),
    };

    JsonFrame {
        data: record.args().to_string(),
        decoder_version: env!("CARGO_PKG_VERSION"),
        host_timestamp,
        level,
        location: Location {
            file: record.file().map(|f| f.to_string()),
            line: record.line(),
            module_path: create_module_path(record.module_path()),
        },
        target_timestamp: record.timestamp().to_string(),
    }
}

fn create_module_path(module_path: Option<&str>) -> Option<ModulePath> {
    let mut path = module_path?.split("::").collect::<Vec<_>>();

    // there need to be at least two elements, the crate and the function
    if path.len() < 2 {
        return None;
    };

    // the last element is the function
    let function = path.pop()?.to_string();
    // the first element is the crate_name
    let crate_name = path.remove(0).to_string();

    Some(ModulePath {
        crate_name,
        modules: path.into_iter().map(|a| a.to_string()).collect(),
        function,
    })
}
