use core::fmt::Write as _;

use crate::{Format, Formatter, Str};

pub use bitflags::bitflags;

pub trait UnsignedInt {}
impl UnsignedInt for u8 {}
impl UnsignedInt for u16 {}
impl UnsignedInt for u32 {}
impl UnsignedInt for u64 {}
impl UnsignedInt for u128 {}

#[cfg(feature = "unstable-test")]
thread_local! {
    static I: core::sync::atomic::AtomicU16 = core::sync::atomic::AtomicU16::new(0);
    static BYTES: core::cell::RefCell<Vec<u8>> = core::cell::RefCell::new(Vec::new());
}

/// For testing purposes
#[cfg(feature = "unstable-test")]
pub fn fetch_string_index() -> u16 {
    I.with(|i| i.load(core::sync::atomic::Ordering::Relaxed))
}

/// For testing purposes
#[cfg(feature = "unstable-test")]
pub fn fetch_add_string_index() -> u16 {
    I.with(|i| i.fetch_add(1, core::sync::atomic::Ordering::Relaxed))
}

/// Get and clear the logged bytes
#[cfg(feature = "unstable-test")]
pub fn fetch_bytes() -> Vec<u8> {
    BYTES.with(|b| core::mem::replace(&mut *b.borrow_mut(), Vec::new()))
}

#[cfg(feature = "unstable-test")]
pub fn acquire() {}

#[cfg(not(feature = "unstable-test"))]
#[inline(never)]
pub fn acquire() {
    extern "Rust" {
        fn _defmt_acquire();
    }
    unsafe { _defmt_acquire() }
}

#[cfg(feature = "unstable-test")]
pub fn flush() {}

#[cfg(not(feature = "unstable-test"))]
#[inline(never)]
pub fn flush() {
    extern "Rust" {
        fn _defmt_acquire();
        fn _defmt_flush();
        fn _defmt_release();
    }
    // SAFETY: ...
    unsafe {
        _defmt_acquire();
        _defmt_flush();
        _defmt_release()
    }
}

#[cfg(feature = "unstable-test")]
pub fn release() {}

#[cfg(not(feature = "unstable-test"))]
#[inline(never)]
pub fn release() {
    extern "Rust" {
        fn _defmt_release();
    }
    unsafe { _defmt_release() }
}

#[cfg(feature = "unstable-test")]
pub fn write(bytes: &[u8]) {
    BYTES.with(|b| b.borrow_mut().extend(bytes))
}

#[cfg(not(feature = "unstable-test"))]
#[inline(never)]
pub fn write(bytes: &[u8]) {
    extern "Rust" {
        fn _defmt_write(bytes: &[u8]);
    }
    unsafe { _defmt_write(bytes) }
}

/// For testing purposes
#[cfg(feature = "unstable-test")]
pub fn timestamp(_fmt: crate::Formatter<'_>) {}

#[cfg(not(feature = "unstable-test"))]
pub fn timestamp(fmt: crate::Formatter<'_>) {
    extern "Rust" {
        fn _defmt_timestamp(_: crate::Formatter<'_>);
    }
    unsafe { _defmt_timestamp(fmt) }
}

/// Returns the interned string at `address`.
pub fn make_istr(address: u16) -> Str {
    Str { address }
}

/// Create a Formatter.
pub fn make_formatter<'a>() -> Formatter<'a> {
    Formatter {
        _phantom: core::marker::PhantomData,
    }
}

mod sealed {
    #[allow(unused_imports)]
    use crate as defmt;
    use crate::{Format, Formatter, Str};

    pub trait Truncate<U> {
        fn truncate(self) -> U;
    }

    impl Truncate<u8> for u8 {
        fn truncate(self) -> u8 {
            self
        }
    }

    impl Truncate<u8> for u16 {
        fn truncate(self) -> u8 {
            self as u8
        }
    }

    impl Truncate<u8> for u32 {
        fn truncate(self) -> u8 {
            self as u8
        }
    }

    impl Truncate<u8> for u64 {
        fn truncate(self) -> u8 {
            self as u8
        }
    }

    impl Truncate<u8> for u128 {
        fn truncate(self) -> u8 {
            self as u8
        }
    }

    // needed so we can call truncate() without having to check whether truncation is necessary first
    impl Truncate<u16> for u16 {
        fn truncate(self) -> u16 {
            self
        }
    }

    impl Truncate<u16> for u32 {
        fn truncate(self) -> u16 {
            self as u16
        }
    }

    impl Truncate<u16> for u64 {
        fn truncate(self) -> u16 {
            self as u16
        }
    }

    impl Truncate<u16> for u128 {
        fn truncate(self) -> u16 {
            self as u16
        }
    }

    // needed so we can call truncate() without having to check whether truncation is necessary first
    impl Truncate<u32> for u32 {
        fn truncate(self) -> u32 {
            self
        }
    }

    impl Truncate<u32> for u64 {
        fn truncate(self) -> u32 {
            self as u32
        }
    }

    impl Truncate<u32> for u128 {
        fn truncate(self) -> u32 {
            self as u32
        }
    }

    // needed so we can call truncate() without having to check whether truncation is necessary first
    impl Truncate<u64> for u64 {
        fn truncate(self) -> u64 {
            self
        }
    }

    impl Truncate<u64> for u128 {
        fn truncate(self) -> u64 {
            self as u64
        }
    }

    // needed so we can call truncate() without having to check whether truncation is necessary first
    impl Truncate<u128> for u128 {
        fn truncate(self) -> u128 {
            self
        }
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct NoneError;

    impl Format for NoneError {
        fn format(&self, _fmt: Formatter) {
            unreachable!();
        }

        fn _format_tag() -> Str {
            defmt_macros::internp!("Unwrap of a None option value")
        }

        fn _format_data(&self) {}
    }

    pub trait IntoResult {
        type Ok;
        type Error;
        fn into_result(self) -> Result<Self::Ok, Self::Error>;
    }

    impl<T> IntoResult for Option<T> {
        type Ok = T;
        type Error = NoneError;

        #[inline]
        fn into_result(self) -> Result<T, NoneError> {
            self.ok_or(NoneError)
        }
    }

    impl<T, E> IntoResult for Result<T, E> {
        type Ok = T;
        type Error = E;

        #[inline]
        fn into_result(self) -> Self {
            self
        }
    }
}

pub fn truncate<T>(x: impl sealed::Truncate<T>) -> T {
    x.truncate()
}

pub fn into_result<T: sealed::IntoResult>(x: T) -> Result<T::Ok, T::Error> {
    x.into_result()
}

/// For testing purposes
#[cfg(feature = "unstable-test")]
pub fn panic() -> ! {
    panic!()
}

#[cfg(not(feature = "unstable-test"))]
pub fn panic() -> ! {
    extern "Rust" {
        fn _defmt_panic() -> !;
    }
    unsafe { _defmt_panic() }
}

/// Implementation detail
pub fn fmt<T: Format + ?Sized>(f: &T) {
    istr(&T::_format_tag());
    f._format_data();
}

/// Implementation detail
pub fn i8(b: &i8) {
    write(&b.to_le_bytes())
}

/// Implementation detail
pub fn i16(b: &i16) {
    write(&b.to_le_bytes())
}

/// Implementation detail
pub fn i32(b: &i32) {
    write(&b.to_le_bytes())
}

/// Implementation detail
pub fn i64(b: &i64) {
    write(&b.to_le_bytes())
}

/// Implementation detail
pub fn i128(b: &i128) {
    write(&b.to_le_bytes())
}

/// Implementation detail
pub fn isize(b: &isize) {
    write(&(*b as i32).to_le_bytes())
}

/// Implementation detail
pub fn fmt_slice<T: Format>(values: &[T]) {
    usize(&values.len());
    istr(&T::_format_tag());
    for value in values {
        value._format_data();
    }
}

/// Implementation detail
pub fn u8(b: &u8) {
    write(&[*b])
}

/// Implementation detail
pub fn u16(b: &u16) {
    write(&b.to_le_bytes())
}

/// Implementation detail
pub fn u32(b: &u32) {
    write(&b.to_le_bytes())
}

/// Implementation detail
pub fn u64(b: &u64) {
    write(&b.to_le_bytes())
}

/// Implementation detail
pub fn u128(b: &u128) {
    write(&b.to_le_bytes())
}

/// Implementation detail
pub fn usize(b: &usize) {
    write(&(*b as u32).to_le_bytes())
}

/// Implementation detail
pub fn f32(b: &f32) {
    write(&f32::to_bits(*b).to_le_bytes())
}

/// Implementation detail
pub fn f64(b: &f64) {
    write(&f64::to_bits(*b).to_le_bytes())
}

/// Implementation detail
pub fn char(b: &char) {
    write(&(*b as u32).to_le_bytes())
}

pub fn str(s: &str) {
    usize(&s.len());
    write(s.as_bytes());
}

pub fn slice(s: &[u8]) {
    usize(&s.len());
    write(s);
}

// NOTE: This is passed `&[u8; N]` – it's just coerced to a slice.
pub fn u8_array(a: &[u8]) {
    write(a);
}

// NOTE: This is passed `&[u8; N]` – it's just coerced to a slice.
pub fn fmt_array<T: Format>(a: &[T]) {
    istr(&T::_format_tag());
    for value in a {
        value._format_data();
    }
}

/// Implementation detail
pub fn istr(s: &Str) {
    write(&s.address.to_le_bytes())
}

/// Implementation detail
pub fn bool(b: &bool) {
    u8(&(*b as u8));
}

/// Implementation detail
pub fn debug(val: &dyn core::fmt::Debug) {
    core::write!(FmtWrite, "{:?}", val).ok();
    write(&[0xff]);
}

/// Implementation detail
pub fn display(val: &dyn core::fmt::Display) {
    core::write!(FmtWrite, "{}", val).ok();
    write(&[0xff]);
}

#[inline(never)]
pub fn header(s: &Str) {
    istr(s);
    timestamp(make_formatter());
}

struct FmtWrite;

impl core::fmt::Write for FmtWrite {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        write(s.as_bytes());
        Ok(())
    }
}
