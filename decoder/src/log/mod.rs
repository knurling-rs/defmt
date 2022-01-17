//! This module provides interoperability utilities between [`defmt`] and the [`log`] crate.
//!
//! If you are implementing a custom defmt decoding tool, this module can make it easier to
//! integrate it with logs produced with the [`log`] crate.
//!
//! [`log`]: https://crates.io/crates/log
//! [`defmt`]: https://crates.io/crates/defmt

mod json_logger;
mod pretty_logger;

use log::{Level, Metadata, Record};
use serde::{Deserialize, Serialize};

use std::fmt;

use self::{
    json_logger::JsonLogger,
    pretty_logger::{PrettyLogger, Printer},
};
use crate::Frame;

pub use json_logger::JsonFrame;

const DEFMT_TARGET_MARKER: &str = "defmt@";

/// Logs a defmt frame using the `log` facade.
pub fn log_defmt(
    frame: &Frame<'_>,
    file: Option<&str>,
    line: Option<u32>,
    module_path: Option<&str>,
) {
    let timestamp = frame
        .display_timestamp()
        .map(|ts| ts.to_string())
        .unwrap_or_default();

    let (level, is_println) = if let Some(level) = frame.level() {
        let level = match level {
            crate::Level::Trace => Level::Trace,
            crate::Level::Debug => Level::Debug,
            crate::Level::Info => Level::Info,
            crate::Level::Warn => Level::Warn,
            crate::Level::Error => Level::Error,
        };
        (level, false)
    } else {
        // If `frame.level()` is `None` then we are inside a `defmt::println!` statement
        (Level::Trace, true)
    };

    let target = format!(
        "{}{}",
        DEFMT_TARGET_MARKER,
        serde_json::to_value(Payload {
            timestamp,
            is_println
        })
        .unwrap()
    );

    log::logger().log(
        &Record::builder()
            .args(format_args!("{}", frame.display_message()))
            .level(level)
            .target(&target)
            .module_path(module_path)
            .file(file)
            .line(line)
            .build(),
    );
}

/// Determines whether `metadata` belongs to a log record produced by [`log_defmt`].
pub fn is_defmt_frame(metadata: &Metadata) -> bool {
    metadata.target().starts_with(DEFMT_TARGET_MARKER)
}

/// A `log` record representing a defmt log frame.
pub struct DefmtRecord<'a> {
    log_record: &'a Record<'a>,
    payload: Payload,
}

#[derive(Deserialize, Serialize)]
struct Payload {
    timestamp: String,
    is_println: bool,
}

impl<'a> DefmtRecord<'a> {
    /// If `record` was produced by [`log_defmt`], returns the corresponding `DefmtRecord`.
    pub fn new(log_record: &'a Record<'a>) -> Option<Self> {
        let target = log_record.metadata().target();
        if let Some(payload) = target.strip_prefix(DEFMT_TARGET_MARKER) {
            let payload = serde_json::from_str(payload).unwrap();
            Some(Self {
                log_record,
                payload,
            })
        } else {
            None
        }
    }

    /// Returns the formatted defmt timestamp.
    pub fn timestamp(&self) -> &str {
        self.payload.timestamp.as_str()
    }

    pub fn level(&self) -> Level {
        self.log_record.level()
    }

    pub fn args(&self) -> &fmt::Arguments<'a> {
        self.log_record.args()
    }

    pub fn module_path(&self) -> Option<&'a str> {
        self.log_record.module_path()
    }

    pub fn file(&self) -> Option<&'a str> {
        self.log_record.file()
    }

    pub fn line(&self) -> Option<u32> {
        self.log_record.line()
    }

    pub fn is_println(&self) -> bool {
        self.payload.is_println
    }

    /// Returns a builder that can format this record for displaying it to the user.
    pub fn printer(&'a self) -> Printer<'a> {
        Printer::new(self)
    }
}

/// Initializes a `log` sink that handles defmt frames.
///
/// Defmt frames will be printed to stdout, other logs to stderr.
///
/// The caller has to provide a `should_log` closure that determines whether a log record should be
/// printed.
///
/// If `always_include_location` is `true`, a second line containing location information will be
/// printed for *all* records, not just for defmt frames (defmt frames always get location info
/// included if it is available, regardless of this setting).
pub fn init_logger(
    always_include_location: bool,
    json: bool,
    should_log: impl Fn(&log::Metadata) -> bool + Sync + Send + 'static,
) {
    log::set_boxed_logger(match json {
        false => PrettyLogger::new(always_include_location, should_log),
        true => JsonLogger::new(should_log),
    })
    .unwrap();
    log::set_max_level(log::LevelFilter::Trace);
}
