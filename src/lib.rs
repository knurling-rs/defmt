//! `defmt`
//!
//! > **PRE-ALPHA PREVIEW** `defmt` wire format has not been finalized yet. When using the
//! > framework make sure you use the *same* "version" (commit hash) for all components (target side
//! > and host side).
//!
//! A highly efficient logging framework that targets resource-constrained devices, like
//! microcontrollers.
//!
//! For more details check the book at <https://defmt.ferrous-systems.com>

#![cfg_attr(not(target_arch = "x86_64"), no_std)]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::mem::MaybeUninit;
use core::ptr::NonNull;

#[doc(hidden)]
pub mod export;
mod impls;
mod leb;
#[cfg(test)]
mod tests;

/// Creates an interned string ([`Str`]) from a string literal.
///
/// This must be called on a string literal, and will allocate the literal in the object file. At
/// runtime, only a small string index is required to refer to the string, represented as the
/// [`Str`] type.
///
/// # Example
///
/// ```
/// let interned = defmt::intern!("long string literal taking up little space");
/// ```
///
/// [`Str`]: struct.Str.html
pub use defmt_macros::intern;

/// Logs data at *debug* level.
pub use defmt_macros::debug;
/// Logs data at *error* level.
pub use defmt_macros::error;
/// Logs data at *info* level.
pub use defmt_macros::info;
/// Logs data at *trace* level.
pub use defmt_macros::trace;
/// Logs data at *warn* level.
pub use defmt_macros::warn;

/// Defines the global defmt logger.
///
/// `#[global_logger]` needs to be put on a unit struct type declaration. This struct has to
/// implement the [`Logger`] trait.
///
/// # Example
///
/// ```
/// use defmt::{Logger, Write, global_logger};
/// use core::ptr::NonNull;
///
/// #[global_logger]
/// struct MyLogger;
///
/// unsafe impl Logger for MyLogger {
///     fn acquire() -> Option<NonNull<dyn Write>> {
/// # todo!()
///         // ...
///     }
///     unsafe fn release(writer: NonNull<dyn Write>) {
/// # todo!()
///         // ...
///     }
/// }
/// ```
///
/// [`Logger`]: trait.Logger.html
pub use defmt_macros::global_logger;

/// Defines the global timestamp provider for defmt.
///
/// Every message logged with defmt will include a timestamp. The function annotated with
/// `#[timestamp]` will be used to obtain this timestamp.
///
/// The `#[timestamp]` attribute needs to be applied to a function with the signature `fn() -> u64`.
/// The returned `u64` is the current timestamp in microseconds.
///
/// Some systems might not have a timer available. In that case, a dummy implementation such as this
/// may be used:
///
/// ```
/// # use defmt_macros::timestamp;
/// #[timestamp]
/// fn dummy_timestamp() -> u64 {
///     0
/// }
/// ```
pub use defmt_macros::timestamp;

#[doc(hidden)]
pub use defmt_macros::winfo;
#[doc(hidden)] // documented as the `Format` trait instead
pub use defmt_macros::Format;

/// Global logger acquire-release mechanism
///
/// # Safety contract
///
/// - `acquire` returns a handle that temporarily *owns* the global logger
/// - `acquire` must return `Some` only once, until the handle is `release`-d
/// - `acquire` is allowed to return a handle per thread or interrupt level
/// - `acquire` is a safe function therefore it must be thread-safe and interrupt-safe
/// - The value returned by `acquire` is not `Send` so it cannot be moved between threads or
/// interrupt handlers
///
/// And, not safety related, `acquire` should never be invoked from user code. The easiest way to
/// ensure this is to implement `Logger` on a *private* `struct` and mark that `struct` as the
/// `#[global_logger]`.
pub unsafe trait Logger {
    /// Returns a handle to the global logger
    ///
    /// For the requirements of the method see the documentation of the `Logger` trait
    fn acquire() -> Option<NonNull<dyn Write>>;

    /// Releases the global logger
    ///
    /// # Safety
    /// `writer` argument must be a value previously returned by `Self::acquire` and not, say,
    /// `NonNull::dangling()`
    unsafe fn release(writer: NonNull<dyn Write>);
}

/// An interned string created via [`intern!`].
///
/// [`intern!`]: macro.intern.html
#[derive(Clone, Copy)]
pub struct Str {
    // 14-bit address
    address: u16,
}

/// Handle to a defmt logger.
pub struct Formatter {
    #[cfg(not(target_arch = "x86_64"))]
    writer: NonNull<dyn Write>,
    #[cfg(target_arch = "x86_64")]
    bytes: Vec<u8>,
    bool_flags: u8, // the current group of consecutive bools
    bools_left: u8, // the number of bits that we can still set in bool_flag
    // whether to omit the tag of a `Format` value
    // this is disabled while formatting a `{:[?]}` value (second element on-wards)
    // this is force-enable while formatting enums
    omit_tag: bool,
}

/// the maximum number of booleans that can be compressed together
const MAX_NUM_BOOL_FLAGS: u8 = 8;

impl Formatter {
    /// Only for testing on x86_64
    #[cfg(target_arch = "x86_64")]
    pub fn new() -> Self {
        Self {
            bytes: vec![],
            bool_flags: 0,
            bools_left: MAX_NUM_BOOL_FLAGS,
            omit_tag: false,
        }
    }

    /// Only for testing on x86_64
    #[cfg(target_arch = "x86_64")]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[doc(hidden)]
    #[cfg(target_arch = "x86_64")]
    pub fn write(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes)
    }

    #[doc(hidden)]
    #[cfg(not(target_arch = "x86_64"))]
    pub fn write(&mut self, bytes: &[u8]) {
        unsafe { self.writer.as_mut().write(bytes) }
    }

    /// Implementation detail
    #[cfg(target_arch = "x86_64")]
    #[doc(hidden)]
    pub unsafe fn from_raw(_: NonNull<dyn Write>) -> Self {
        unreachable!()
    }

    /// Implementation detail
    #[cfg(not(target_arch = "x86_64"))]
    #[doc(hidden)]
    pub unsafe fn from_raw(writer: NonNull<dyn Write>) -> Self {
        Self {
            writer,
            bool_flags: 0,
            bools_left: MAX_NUM_BOOL_FLAGS,
            omit_tag: false,
        }
    }

    /// Implementation detail
    #[cfg(target_arch = "x86_64")]
    #[doc(hidden)]
    pub unsafe fn into_raw(self) -> NonNull<dyn Write> {
        unreachable!()
    }

    /// Implementation detail
    #[cfg(not(target_arch = "x86_64"))]
    #[doc(hidden)]
    pub unsafe fn into_raw(self) -> NonNull<dyn Write> {
        self.writer
    }

    // TODO turn these public methods in `export` free functions
    /// Implementation detail
    #[doc(hidden)]
    pub fn fmt(&mut self, f: &impl Format, omit_tag: bool) {
        let old_omit_tag = self.omit_tag;
        if omit_tag {
            self.omit_tag = true;
        }

        f.format(self);

        if omit_tag {
            // restore
            self.omit_tag = old_omit_tag;
        }
    }

    /// Implementation detail
    #[doc(hidden)]
    pub fn needs_tag(&self) -> bool {
        !self.omit_tag
    }

    /// Implementation detail
    #[doc(hidden)]
    pub fn with_tag(&mut self, f: impl FnOnce(&mut Self)) {
        let omit_tag = self.omit_tag;
        self.omit_tag = false;
        f(self);
        // restore
        self.omit_tag = omit_tag;
    }

    /// Implementation detail
    /// leb64-encode `x` and write it to self.bytes
    #[doc(hidden)]
    pub fn leb64(&mut self, x: u64) {
        // FIXME: Avoid 64-bit arithmetic on 32-bit systems. This should only be used for
        // pointer-sized values.
        let mut buf: [u8; 10] = unsafe { MaybeUninit::uninit().assume_init() };
        let i = unsafe { leb::leb64(x, &mut buf) };
        self.write(unsafe { buf.get_unchecked(..i) })
    }

    /// Implementation detail
    #[doc(hidden)]
    pub fn i8(&mut self, b: &i8) {
        self.write(&b.to_le_bytes())
    }

    /// Implementation detail
    #[doc(hidden)]
    pub fn i16(&mut self, b: &i16) {
        self.write(&b.to_le_bytes())
    }

    /// Implementation detail
    #[doc(hidden)]
    pub fn i32(&mut self, b: &i32) {
        self.write(&b.to_le_bytes())
    }

    /// Implementation detail
    #[doc(hidden)]
    pub fn i64(&mut self, b: &i64) {
        self.write(&b.to_le_bytes())
    }

    /// Implementation detail
    #[doc(hidden)]
    pub fn isize(&mut self, b: &isize) {
        // Zig-zag encode the signed value.
        self.leb64(leb::zigzag_encode(*b as i64));
    }

    /// Implementation detail
    #[doc(hidden)]
    pub fn fmt_slice(&mut self, values: &[impl Format]) {
        self.leb64(values.len() as u64);
        let mut is_first = true;
        for value in values {
            let omit_tag = !is_first;
            self.fmt(value, omit_tag);
            is_first = false;
        }
    }

    // TODO remove
    /// Implementation detail
    #[doc(hidden)]
    pub fn prim(&mut self, s: &Str) {
        self.write(&[s.address as u8])
    }

    /// Implementation detail
    #[doc(hidden)]
    pub fn u8(&mut self, b: &u8) {
        self.write(&[*b])
    }

    /// Implementation detail
    #[doc(hidden)]
    pub fn u16(&mut self, b: &u16) {
        self.write(&b.to_le_bytes())
    }

    /// Implementation detail
    #[doc(hidden)]
    pub fn u24(&mut self, b: &u32) {
        self.write(&b.to_le_bytes()[..3])
    }

    /// Implementation detail
    #[doc(hidden)]
    pub fn u32(&mut self, b: &u32) {
        self.write(&b.to_le_bytes())
    }

    /// Implementation detail
    #[doc(hidden)]
    pub fn u64(&mut self, b: &u64) {
        self.write(&b.to_le_bytes())
    }

    /// Implementation detail
    #[doc(hidden)]
    pub fn usize(&mut self, b: &usize) {
        self.leb64(*b as u64);
    }

    /// Implementation detail
    #[doc(hidden)]
    pub fn f32(&mut self, b: &f32) {
        self.write(&f32::to_bits(*b).to_le_bytes())
    }

    #[doc(hidden)]
    pub fn str(&mut self, s: &str) {
        self.leb64(s.len() as u64);
        self.write(s.as_bytes());
    }

    #[doc(hidden)]
    pub fn slice(&mut self, s: &[u8]) {
        self.leb64(s.len() as u64);
        self.write(s);
    }

    // NOTE: This is passed `&[u8; N]` – it's just coerced to a slice.
    #[doc(hidden)]
    pub fn u8_array(&mut self, a: &[u8]) {
        self.write(a);
    }

    // NOTE: This is passed `&[u8; N]` – it's just coerced to a slice.
    #[doc(hidden)]
    pub fn fmt_array(&mut self, a: &[impl Format]) {
        let mut is_first = true;
        for value in a {
            let omit_tag = !is_first;
            self.fmt(value, omit_tag);
            is_first = false;
        }
    }

    /// Implementation detail
    #[doc(hidden)]
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
        let b_u8 = *b as u8;
        // set n'th bool flag
        self.bool_flags = (self.bool_flags << 1) | b_u8;
        self.bools_left -= 1;

        // if we've filled max compression space, flush and begin anew
        if self.bools_left == 0 {
            self.flush_and_reset_bools();
        }
    }

    /// The last pass in a formatting run: clean up & flush leftovers
    pub fn finalize(&mut self) {
        if self.bools_left < MAX_NUM_BOOL_FLAGS {
            // there are bools in compression that haven't been flushed yet
            self.flush_and_reset_bools();
        }
    }

    fn flush_and_reset_bools(&mut self) {
        let flags = self.bool_flags;
        self.u8(&flags);
        self.bools_left = MAX_NUM_BOOL_FLAGS;
        self.bool_flags = 0;
    }
}

/// Trait for defmt logging targets.
pub trait Write {
    /// Writes `bytes` to the destination.
    ///
    /// This will be called by the defmt logging macros to transmit encoded data. The write
    /// operation must not fail.
    fn write(&mut self, bytes: &[u8]);
}

/// Derivable trait for defmt output.
///
/// This trait is used by the `{:?}` format specifier and can format a wide range of types.
/// User-defined types can `#[derive(Format)]` to get an auto-generated implementation of this
/// trait.
///
/// **Note**: The implementation of `#[derive(Format)]` assumes that no builtin types are shadowed
/// (for example by defining a `struct u8;`). This allows it to represent them more compactly.
///
/// # Example
///
/// It is required to `#[derive]` implementations of this trait:
///
/// ```
/// use defmt::Format;
///
/// #[derive(Format)]
/// struct Header {
///     source: u8,
///     destination: u8,
///     sequence: u16,
/// }
/// ```
pub trait Format {
    /// Writes the defmt representation of `self` to `fmt`.
    fn format(&self, fmt: &mut Formatter);
}

#[export_name = "__defmt_default_timestamp"]
fn default_timestamp() -> u64 {
    0
}
