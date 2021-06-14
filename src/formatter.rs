use core::fmt::{self, Write as _};

use crate::{export, Format};

/// Handle to a defmt logger.
pub struct Formatter<'a> {
    /// Keep the formatter alive
    #[doc(hidden)]
    pub inner: &'a mut InternalFormatter,
}

#[doc(hidden)]
pub struct InternalFormatter {}

#[doc(hidden)]
impl InternalFormatter {
    pub fn write(&mut self, bytes: &[u8]) {
        export::write(bytes)
    }

    /// Implementation detail
    ///
    /// # Safety
    ///
    /// Must only be called when the current execution context has acquired the logger with [Logger::acquire]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    // TODO turn these public methods in `export` free functions
    /// Implementation detail
    pub fn fmt<T: Format>(&mut self, f: &T) {
        self.tag(T::_format_tag());
        let formatter = Formatter { inner: self };
        f._format_data(formatter);
    }

    /// Implementation detail
    pub fn tag(&mut self, b: u16) {
        self.write(&b.to_le_bytes())
    }

    /// Implementation detail
    pub fn i8(&mut self, b: &i8) {
        self.write(&b.to_le_bytes())
    }

    /// Implementation detail
    pub fn i16(&mut self, b: &i16) {
        self.write(&b.to_le_bytes())
    }

    /// Implementation detail
    pub fn i32(&mut self, b: &i32) {
        self.write(&b.to_le_bytes())
    }

    /// Implementation detail
    pub fn i64(&mut self, b: &i64) {
        self.write(&b.to_le_bytes())
    }

    /// Implementation detail
    pub fn i128(&mut self, b: &i128) {
        self.write(&b.to_le_bytes())
    }

    /// Implementation detail
    pub fn isize(&mut self, b: &isize) {
        self.write(&(*b as i32).to_le_bytes())
    }

    /// Implementation detail
    pub fn fmt_slice(&mut self, values: &[impl Format]) {
        self.usize(&values.len());
        for value in values {
            self.fmt(value);
        }
    }

    // TODO remove
    /// Implementation detail
    pub fn prim(&mut self, s: &Str) {
        self.write(&[s.address as u8])
    }

    /// Implementation detail
    pub fn u8(&mut self, b: &u8) {
        self.write(&[*b])
    }

    /// Implementation detail
    pub fn u16(&mut self, b: &u16) {
        self.write(&b.to_le_bytes())
    }

    /// Implementation detail
    pub fn u24(&mut self, b: &u32) {
        self.write(&b.to_le_bytes()[..3])
    }

    /// Implementation detail
    pub fn u32(&mut self, b: &u32) {
        self.write(&b.to_le_bytes())
    }

    /// Implementation detail
    pub fn u64(&mut self, b: &u64) {
        self.write(&b.to_le_bytes())
    }

    /// Implementation detail
    pub fn u128(&mut self, b: &u128) {
        self.write(&b.to_le_bytes())
    }

    /// Implementation detail
    pub fn usize(&mut self, b: &usize) {
        self.write(&(*b as u32).to_le_bytes())
    }

    /// Implementation detail
    pub fn f32(&mut self, b: &f32) {
        self.write(&f32::to_bits(*b).to_le_bytes())
    }

    /// Implementation detail
    pub fn f64(&mut self, b: &f64) {
        self.write(&f64::to_bits(*b).to_le_bytes())
    }

    pub fn str(&mut self, s: &str) {
        self.usize(&s.len());
        self.write(s.as_bytes());
    }

    pub fn slice(&mut self, s: &[u8]) {
        self.usize(&s.len());
        self.write(s);
    }

    // NOTE: This is passed `&[u8; N]` – it's just coerced to a slice.
    pub fn u8_array(&mut self, a: &[u8]) {
        self.write(a);
    }

    // NOTE: This is passed `&[u8; N]` – it's just coerced to a slice.
    pub fn fmt_array(&mut self, a: &[impl Format]) {
        for value in a {
            self.fmt(value);
        }
    }

    /// Implementation detail
    pub fn istr(&mut self, s: &Str) {
        self.write(&s.address.to_le_bytes())
    }

    /// Implementation detail
    pub fn bool(&mut self, b: &bool) {
        self.u8(&(*b as u8));
    }

    /// Implementation detail
    pub fn debug(&mut self, val: &dyn core::fmt::Debug) {
        core::write!(FmtWrite { fmt: self }, "{:?}", val).ok();
        self.write(&[0xff]);
    }

    /// Implementation detail
    pub fn display(&mut self, val: &dyn core::fmt::Display) {
        core::write!(FmtWrite { fmt: self }, "{}", val).ok();
        self.write(&[0xff]);
    }

    #[inline(never)]
    pub fn header(&mut self, s: &Str) {
        self.istr(s);
        export::timestamp(Formatter { inner: self });
    }
}

/// An interned string created via [`intern!`].
///
/// [`intern!`]: macro.intern.html
#[derive(Clone, Copy)]
pub struct Str {
    /// 14-bit address
    pub(crate) address: u16,
}

struct FmtWrite<'a> {
    fmt: &'a mut InternalFormatter,
}

impl fmt::Write for FmtWrite<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.fmt.write(s.as_bytes());
        Ok(())
    }
}
