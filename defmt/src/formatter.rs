use crate::{get_global_log_level, IdRanges, LogLevel};
use core::marker::PhantomData;

/// Handle to a defmt logger.
#[derive(Copy, Clone)]
pub struct Formatter<'a> {
    pub(crate) _phantom: PhantomData<&'a ()>,
}

/// An interned string created via [`intern!`].
///
/// [`intern!`]: macro.intern.html
#[derive(Clone, Copy)]
pub struct Str {
    /// 16-bit address
    pub(crate) address: u16,
}

impl Str {
    /// If the interned string is a log message, returns whether its level is above the global log level.
    /// If the interned string is not a log message, returns `false`.
    /// See [`set_global_log_level`] to change the log level
    pub(crate) fn level_above_global_log_level(&self) -> bool {
        let ranges = IdRanges::get();
        if self.address >= ranges.trace.start && self.address < ranges.error.end {
            let min_id = match get_global_log_level() {
                LogLevel::Trace => ranges.trace.start,
                LogLevel::Debug => ranges.debug.start,
                LogLevel::Info => ranges.info.start,
                LogLevel::Warn => ranges.warn.start,
                LogLevel::Error => ranges.error.start,
            };
            self.address >= min_id
        } else {
            false
        }
    }
}
