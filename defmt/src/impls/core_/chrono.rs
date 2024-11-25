use super::*;
extern crate chrono;
use chrono::{
    DateTime, Datelike, FixedOffset, Months, NaiveDate, NaiveDateTime, NaiveTime, OutOfRange,
    TimeDelta, TimeZone, Timelike, Utc,
};

impl<Tz: TimeZone> Format for DateTime<Tz> {
    fn format(&self, fmt: Formatter) {
        crate::write!(
            fmt,
            "{:04}-{:02}-{:02}",
            self.year(),
            self.month(),
            self.day()
        );
    }
}

impl Format for FixedOffset {
    fn format(&self, fmt: Formatter) {
        crate::write!(fmt, "FixedOffset({})", self.local_minus_utc());
    }
}

impl Format for Months {
    fn format(&self, fmt: Formatter) {
        crate::write!(fmt, "Months({=u32})", self.as_u32());
    }
}

impl Format for NaiveDate {
    fn format(&self, fmt: Formatter) {
        crate::write!(
            fmt,
            "{:04}-{:02}-{:02}",
            self.year(),
            self.month(),
            self.day()
        );
    }
}

impl Format for NaiveDateTime {
    fn format(&self, fmt: Formatter) {
        crate::write!(fmt, "{}T{}", self.date(), self.time());
    }
}

impl Format for NaiveTime {
    fn format(&self, fmt: Formatter) {
        crate::write!(fmt, "{}:{}:{}", self.hour(), self.minute(), self.second());
    }
}

impl Format for OutOfRange {
    fn format(&self, fmt: Formatter) {
        crate::write!(fmt, "out of range");
    }
}

impl Format for TimeDelta {
    fn format(&self, fmt: Formatter) {
        crate::write!(
            fmt,
            "TimeDelta {{ secs: {=i64}, nanos: {=i32} }}",
            self.num_seconds(),
            self.subsec_nanos()
        );
    }
}

impl Format for Utc {
    fn format(&self, fmt: Formatter) {
        crate::write!(fmt, "UTC");
    }
}
