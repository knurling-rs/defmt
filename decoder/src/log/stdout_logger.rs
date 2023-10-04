use log::{Log, Metadata, Record as LogRecord};
use std::io::{self, StderrLock, StdoutLock, Write};

use super::{
    format::{DefmtFormatter, HostFormatter},
    DefmtRecord,
};

pub(crate) struct StdoutLogger {
    formatter: DefmtFormatter,
    host_formatter: HostFormatter,
    should_log: Box<dyn Fn(&Metadata) -> bool + Sync + Send>,
}

impl Log for StdoutLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        (self.should_log)(metadata)
    }

    fn log(&self, record: &LogRecord) {
        if !self.enabled(record.metadata()) {
            return;
        }

        match DefmtRecord::new(record) {
            Some(record) => {
                // defmt goes to stdout, since it's the primary output produced by this tool.
                let sink = io::stdout().lock();
                if record.level().is_some() {
                    self.print_defmt_record(record, sink);
                } else {
                    self.print_defmt_record_without_format(record, sink);
                }
            }
            None => {
                // non-defmt logs go to stderr
                let sink = io::stderr().lock();
                self.print_host_record(record, sink);
            }
        }
    }

    fn flush(&self) {}
}

impl StdoutLogger {
    pub fn new(
        formatter: DefmtFormatter,
        host_formatter: HostFormatter,
        should_log: impl Fn(&Metadata) -> bool + Sync + Send + 'static,
    ) -> Box<Self> {
        Box::new(Self::new_unboxed(formatter, host_formatter, should_log))
    }

    pub fn new_unboxed(
        formatter: DefmtFormatter,
        host_formatter: HostFormatter,
        should_log: impl Fn(&Metadata) -> bool + Sync + Send + 'static,
    ) -> Self {
        Self {
            formatter,
            host_formatter,
            should_log: Box::new(should_log),
        }
    }

    fn print_defmt_record(&self, record: DefmtRecord, mut sink: StdoutLock) {
        let s = self.formatter.format(&record);
        writeln!(sink, "{s}").ok();
    }

    pub(super) fn print_defmt_record_without_format(
        &self,
        record: DefmtRecord,
        mut sink: StdoutLock,
    ) {
        let s = record.args().to_string();
        writeln!(sink, "{s}").ok();
    }

    pub(super) fn print_host_record(&self, record: &LogRecord, mut sink: StderrLock) {
        let s = self.host_formatter.format(record);
        writeln!(sink, "{s}").ok();
    }
}
