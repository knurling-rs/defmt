use core::fmt::{self, Write as _};

use crate::{export, leb, Format};

/// Handle to a defmt logger.
pub struct Formatter<'a> {
    /// Keep the formatter alive
    #[doc(hidden)]
    pub inner: &'a mut InternalFormatter,
}

#[doc(hidden)]
pub struct InternalFormatter {
    #[cfg(feature = "unstable-test")]
    bytes: Vec<u8>,
    /// Whether to omit the tag of a `Format` value
    ///
    /// * this is disabled while formatting a `{:[?]}` value (second element on-wards)
    /// * this is force-enabled while formatting enums
    omit_tag: bool,
}

#[doc(hidden)]
impl InternalFormatter {
    #[cfg(not(feature = "unstable-test"))]
    pub fn write(&mut self, bytes: &[u8]) {
        export::write(bytes)
    }

    /// Implementation detail
    ///
    /// # Safety
    ///
    /// Must only be called when the current execution context has acquired the logger with [Logger::acquire]
    #[cfg(not(feature = "unstable-test"))]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { omit_tag: false }
    }

    // TODO turn these public methods in `export` free functions
    /// Implementation detail
    pub fn fmt(&mut self, f: &impl Format, omit_tag: bool) {
        let old_omit_tag = self.omit_tag;
        if omit_tag {
            self.omit_tag = true;
        }

        let formatter = Formatter { inner: self };
        f.format(formatter);

        if omit_tag {
            // restore
            self.omit_tag = old_omit_tag;
        }
    }

    /// Implementation detail
    pub fn needs_tag(&self) -> bool {
        !self.omit_tag
    }

    /// Implementation detail
    pub fn with_tag(&mut self, f: impl FnOnce(Formatter)) {
        let omit_tag = self.omit_tag;
        self.omit_tag = false;

        let formatter = Formatter { inner: self };
        f(formatter);
        // restore
        self.omit_tag = omit_tag;
    }

    /// Implementation detail
    /// leb64-encode `x` and write it to self.bytes
    pub fn leb64(&mut self, x: usize) {
        let mut buf: [u8; 10] = [0; 10];
        let i = leb::leb64(x, &mut buf);
        self.write(&buf[..i])
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
        // Zig-zag encode the signed value.
        self.leb64(leb::zigzag_encode(*b));
    }

    /// Implementation detail
    pub fn fmt_slice(&mut self, values: &[impl Format]) {
        self.leb64(values.len());
        let mut is_first = true;
        for value in values {
            let omit_tag = !is_first;
            self.fmt(value, omit_tag);
            is_first = false;
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
        self.leb64(*b);
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
        self.leb64(s.len());
        self.write(s.as_bytes());
    }

    pub fn slice(&mut self, s: &[u8]) {
        self.leb64(s.len());
        self.write(s);
    }

    // NOTE: This is passed `&[u8; N]` – it's just coerced to a slice.
    pub fn u8_array(&mut self, a: &[u8]) {
        self.write(a);
    }

    // NOTE: This is passed `&[u8; N]` – it's just coerced to a slice.
    pub fn fmt_array(&mut self, a: &[impl Format]) {
        let mut is_first = true;
        for value in a {
            let omit_tag = !is_first;
            self.fmt(value, omit_tag);
            is_first = false;
        }
    }

    /// Implementation detail
    pub fn istr(&mut self, s: &Str) {
        // LEB128 encoding
        if s.address < 128 {
            self.write(&[s.address as u8])
        } else {
            self.write(&[s.address as u8 | (1 << 7), (s.address >> 7) as u8])
        }
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

#[doc(hidden)]
#[cfg(feature = "unstable-test")]
impl super::InternalFormatter {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            bytes: vec![],
            omit_tag: false,
        }
    }

    pub fn bytes(&mut self) -> &[u8] {
        &self.bytes
    }

    pub fn write(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes)
    }
}
