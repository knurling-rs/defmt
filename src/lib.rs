//! A highly efficient logging framework that targets resource-constrained devices, like
//! microcontrollers.
//!
//! Check out the defmt book at <https://defmt.ferrous-systems.com> for more information about how
//! to use it.
//!
//! # Compatibility
//!
//! The `defmt` wire format might change between major versions. Attempting to read a defmt stream
//! with an incompatible version will result in an error. This means that you have to update both
//! the host and target side if a breaking change in defmt is released.

#![cfg_attr(not(target_arch = "x86_64"), no_std)]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::{mem::MaybeUninit, ptr::NonNull};

#[doc(hidden)]
pub mod export;
mod impls;
mod leb;
#[cfg(test)]
mod tests;

/// Just like the [`core::assert!`] macro but `defmt` is used to log the panic message
///
/// [`core::assert!`]: https://doc.rust-lang.org/core/macro.assert.html
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::assert_ as assert;

/// Just like the [`core::assert_eq!`] macro but `defmt` is used to log the panic message
///
/// [`core::assert_eq!`]: https://doc.rust-lang.org/core/macro.assert_eq.html
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::assert_eq_ as assert_eq;

/// Just like the [`core::assert_ne!`] macro but `defmt` is used to log the panic message
///
/// [`core::assert_ne!`]: https://doc.rust-lang.org/core/macro.assert_ne.html
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::assert_ne_ as assert_ne;

/// Just like the [`core::debug_assert!`] macro but `defmt` is used to log the panic message
///
/// [`core::debug_assert!`]: https://doc.rust-lang.org/core/macro.debug_assert.html
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::debug_assert_ as debug_assert;

/// Just like the [`core::debug_assert_eq!`] macro but `defmt` is used to log the panic message
///
/// [`core::debug_assert_eq!`]: https://doc.rust-lang.org/core/macro.debug_assert_eq.html
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::debug_assert_eq_ as debug_assert_eq;

/// Just like the [`core::debug_assert_ne!`] macro but `defmt` is used to log the panic message
///
/// [`core::debug_assert_ne!`]: https://doc.rust-lang.org/core/macro.debug_assert_ne.html
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::debug_assert_ne_ as debug_assert_ne;

/// Just like the [`core::unreachable!`] macro but `defmt` is used to log the panic message
///
/// [`core::unreachable!`]: https://doc.rust-lang.org/core/macro.unreachable.html
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::unreachable_ as unreachable;

/// Just like the [`core::todo!`] macro but `defmt` is used to log the panic message
///
/// [`core::todo!`]: https://doc.rust-lang.org/core/macro.todo.html
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::todo_ as todo;

/// Just like the [`core::unimplemented!`] macro but `defmt` is used to log the panic message
///
/// [`core::unimplemented!`]: https://doc.rust-lang.org/core/macro.unimplemented.html
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::todo_ as unimplemented;

/// Just like the [`core::panic!`] macro but `defmt` is used to log the panic message
///
/// [`core::panic!`]: https://doc.rust-lang.org/core/macro.panic.html
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::panic_ as panic;

/// todo
///
/// If used, the format string must follow the defmt syntax (documented in [the manual])
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::unwrap;

/// Overrides the panicking behavior of `defmt::panic!`
///
/// By default, `defmt::panic!` calls `core::panic!` after logging the panic message using `defmt`.
/// This can result in the panic message being printed twice in some cases. To avoid that issue use
/// this macro. See [the manual] for details.
///
/// [the manual]: https://defmt.ferrous-systems.com/panic.html
pub use defmt_macros::panic_handler;

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
///
/// Please refer to [the manual] for documentation on the syntax.
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::debug;
/// Logs data at *error* level.
///
/// Please refer to [the manual] for documentation on the syntax.
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::error;
/// Logs data at *info* level.
///
/// Please refer to [the manual] for documentation on the syntax.
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::info;
/// Logs data at *trace* level.
///
/// Please refer to [the manual] for documentation on the syntax.
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::trace;
/// Logs data at *warn* level.
///
/// Please refer to [the manual] for documentation on the syntax.
///
/// [the manual]: https://defmt.ferrous-systems.com/macros.html
pub use defmt_macros::warn;

/// Writes formatted data to a [`Formatter`].
///
/// [`Formatter`]: struct.Formatter.html
pub use defmt_macros::write;

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
/// If no crate defines a `#[timestamp]` function, defmt will default to the following dummy
/// implementation:
///
/// ```
/// # use defmt_macros::timestamp;
/// #[timestamp]
/// fn dummy_timestamp() -> u64 {
///     0
/// }
/// ```
pub use defmt_macros::timestamp;

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
    /// Whether the `write!` macro was called in the current `Format` impl. Used to prevent calling
    /// it twice.
    /// FIXME: Use a dedicated tag for `write!` invocations, allow calling it multiple times, and
    /// remove this.
    called_write_macro: bool,
}

/// the maximum number of booleans that can be compressed together
const MAX_NUM_BOOL_FLAGS: u8 = 8;

#[doc(hidden)]
impl Formatter {
    /// Only for testing on x86_64
    #[cfg(target_arch = "x86_64")]
    pub fn new() -> Self {
        Self {
            bytes: vec![],
            bool_flags: 0,
            bools_left: MAX_NUM_BOOL_FLAGS,
            omit_tag: false,
            called_write_macro: false,
        }
    }

    /// Only for testing on x86_64
    #[cfg(target_arch = "x86_64")]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[cfg(target_arch = "x86_64")]
    pub fn write(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes)
    }

    #[cfg(not(target_arch = "x86_64"))]
    pub fn write(&mut self, bytes: &[u8]) {
        unsafe { self.writer.as_mut().write(bytes) }
    }

    /// Implementation detail
    #[cfg(not(target_arch = "x86_64"))]
    pub unsafe fn from_raw(writer: NonNull<dyn Write>) -> Self {
        Self {
            writer,
            bool_flags: 0,
            bools_left: MAX_NUM_BOOL_FLAGS,
            omit_tag: false,
            called_write_macro: false,
        }
    }

    /// Implementation detail
    #[cfg(not(target_arch = "x86_64"))]
    pub unsafe fn into_raw(self) -> NonNull<dyn Write> {
        self.writer
    }

    // TODO turn these public methods in `export` free functions
    /// Implementation detail
    pub fn fmt(&mut self, f: &impl Format, omit_tag: bool) {
        let old_omit_tag = self.omit_tag;
        let old_called_write_macro = self.called_write_macro;
        if omit_tag {
            self.omit_tag = true;
        }
        self.called_write_macro = false;

        f.format(self);

        self.called_write_macro = old_called_write_macro;
        if omit_tag {
            // restore
            self.omit_tag = old_omit_tag;
        }
    }

    pub fn write_macro_start(&mut self) {
        if self.called_write_macro {
            core::panic!("`defmt::write!` may only be called once in a `Format` impl");
        }

        self.called_write_macro = true;
    }

    /// Implementation detail
    pub fn needs_tag(&self) -> bool {
        !self.omit_tag
    }

    /// Implementation detail
    pub fn with_tag(&mut self, f: impl FnOnce(&mut Self)) {
        let omit_tag = self.omit_tag;
        self.omit_tag = false;
        f(self);
        // restore
        self.omit_tag = omit_tag;
    }

    /// Implementation detail
    /// leb64-encode `x` and write it to self.bytes
    pub fn leb64(&mut self, x: u64) {
        // FIXME: Avoid 64-bit arithmetic on 32-bit systems. This should only be used for
        // pointer-sized values.
        let mut buf: [u8; 10] = unsafe { MaybeUninit::uninit().assume_init() };
        let i = unsafe { leb::leb64(x, &mut buf) };
        self.write(unsafe { buf.get_unchecked(..i) })
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
    pub fn isize(&mut self, b: &isize) {
        // Zig-zag encode the signed value.
        self.leb64(leb::zigzag_encode(*b as i64));
    }

    /// Implementation detail
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
    pub fn usize(&mut self, b: &usize) {
        self.leb64(*b as u64);
    }

    /// Implementation detail
    pub fn f32(&mut self, b: &f32) {
        self.write(&f32::to_bits(*b).to_le_bytes())
    }

    pub fn str(&mut self, s: &str) {
        self.leb64(s.len() as u64);
        self.write(s.as_bytes());
    }

    pub fn slice(&mut self, s: &[u8]) {
        self.leb64(s.len() as u64);
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

    #[inline(never)]
    pub fn header(&mut self, s: &Str) {
        self.istr(s);
        self.leb64(export::timestamp())
    }
}

// these need to be in a separate module or `unreachable!` will end up calling `defmt::panic` and
// this will not compile
// (using `core::unreachable!` instead of `unreachable!` doesn't help)
#[cfg(target_arch = "x86_64")]
mod x86_64 {
    use core::ptr::NonNull;

    use super::Write;

    #[doc(hidden)]
    impl super::Formatter {
        /// Implementation detail
        pub unsafe fn from_raw(_: NonNull<dyn Write>) -> Self {
            unreachable!()
        }

        /// Implementation detail
        pub unsafe fn into_raw(self) -> NonNull<dyn Write> {
            unreachable!()
        }
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

#[export_name = "__defmt_default_panic"]
fn default_panic() -> ! {
    core::panic!()
}
