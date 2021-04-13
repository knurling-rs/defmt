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

#![cfg_attr(not(feature = "unstable-test"), no_std)]
// NOTE if you change this URL you'll also need to update all other crates in this repo
#![doc(html_logo_url = "https://knurling.ferrous-systems.com/knurling_logo_light_text.svg")]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "unstable-test")]
use crate as defmt;

use core::fmt::Write as _;
use core::{fmt, ptr::NonNull};

#[doc(hidden)]
pub mod export;
mod impls;
mod leb;
#[cfg(all(test, feature = "unstable-test"))]
mod tests;
#[cfg(all(test, not(feature = "unstable-test")))]
compile_error!(
    "to run unit tests enable the `unstable-test` feature, e.g. `cargo t --features unstable-test`"
);

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

/// Unwraps an `Option` or `Result`, panicking if it is `None` or `Err`.
///
/// This macro is roughly equivalent to `{Option,Result}::{expect,unwrap}` but invocation looks
/// a bit different because this is a macro and not a method. The other difference is that
/// `unwrap!`-ing a `Result<T, E>` value requires that the error type `E` implements the `Format`
/// trait
///
/// The following snippet shows the differences between core's unwrap method and defmt's unwrap
/// macro:
///
/// ```
/// use defmt::unwrap;
///
/// # let option = Some(());
/// let x = option.unwrap();
/// let x = unwrap!(option);
///
/// # let result = Ok::<(), ()>(());
/// let x = result.unwrap();
/// let x = unwrap!(result);
///
/// # let value = result;
/// let x = value.expect("text");
/// let x = unwrap!(value, "text");
///
/// # let arg = ();
/// let x = value.expect(&format!("text {:?}", arg));
/// let x = unwrap!(value, "text {:?}", arg); // arg must be implement `Format`
/// ```
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
///
/// # Inter-operation with built-in attributes
///
/// This attribute cannot be used together with the `export_name` or `no_mangle` attributes
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
/// This macro can be used to attach a timestamp or other data to every defmt message. Its syntax
/// works exactly like the logging macros, except that no local variables can be accessed and the
/// macro should be placed in a module instead of a function.
///
/// `timestamp!` must only be used once across the crate graph.
///
/// If no crate defines a timestamp, no timestamp will be included in the logged messages.
///
/// # Examples
///
/// ```
/// # use core::sync::atomic::{AtomicU32, Ordering};
///
/// static COUNT: AtomicU32 = AtomicU32::new(0);
/// defmt::timestamp!("{=u32:µs}", COUNT.fetch_add(1, Ordering::Relaxed));
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

#[doc(hidden)]
pub struct InternalFormatter {
    #[cfg(not(feature = "unstable-test"))]
    writer: NonNull<dyn Write>,
    #[cfg(feature = "unstable-test")]
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

/// Handle to a defmt logger.
pub struct Formatter<'a> {
    /// Keep the formatter alive
    #[doc(hidden)]
    pub inner: &'a mut InternalFormatter,
}

#[doc(hidden)]
impl InternalFormatter {
    /// Only for testing
    #[cfg(feature = "unstable-test")]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            bytes: vec![],
            bool_flags: 0,
            bools_left: MAX_NUM_BOOL_FLAGS,
            omit_tag: false,
        }
    }

    /// Only for testing
    #[cfg(feature = "unstable-test")]
    pub fn bytes(&mut self) -> &[u8] {
        self.finalize();
        &self.bytes
    }

    #[cfg(feature = "unstable-test")]
    pub fn write(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes)
    }

    #[cfg(not(feature = "unstable-test"))]
    pub fn write(&mut self, bytes: &[u8]) {
        unsafe { self.writer.as_mut().write(bytes) }
    }

    /// Implementation detail
    /// # Safety
    /// `writer` is `Copy` but the returned type is a singleton. Calling this function should not
    /// break the singleton invariant (one should not create more than one instance of
    /// `InternalFormatter`)
    #[cfg(not(feature = "unstable-test"))]
    pub unsafe fn from_raw(writer: NonNull<dyn Write>) -> Self {
        Self {
            writer,
            bool_flags: 0,
            bools_left: MAX_NUM_BOOL_FLAGS,
            omit_tag: false,
        }
    }

    /// Implementation detail
    #[cfg(not(feature = "unstable-test"))]
    pub fn into_raw(self) -> NonNull<dyn Write> {
        self.writer
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
        let b_u8 = *b as u8;
        // set n'th bool flag
        self.bool_flags = (self.bool_flags << 1) | b_u8;
        self.bools_left -= 1;

        // if we've filled max compression space, flush and begin anew
        if self.bools_left == 0 {
            self.flush_and_reset_bools();
        }
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
        export::timestamp(Formatter { inner: self });
    }
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

// these need to be in a separate module or `unreachable!` will end up calling `defmt::panic` and
// this will not compile
// (using `core::unreachable!` instead of `unreachable!` doesn't help)
#[cfg(feature = "unstable-test")]
mod test_only {
    use core::ptr::NonNull;

    use super::Write;

    #[doc(hidden)]
    impl super::InternalFormatter {
        /// Implementation detail
        ///
        /// # Safety
        ///
        /// This is always safe to call and will panic. It only exists to match the non-test API.
        pub unsafe fn from_raw(_: NonNull<dyn Write>) -> Self {
            unreachable!()
        }

        /// Implementation detail
        ///
        /// # Safety
        ///
        /// This is always safe to call and will panic. It only exists to match the non-test API.
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
    ///
    /// Note that a call to `write` does *not* correspond to a defmt logging macro invocation. A
    /// single `defmt::info!` call can result in an arbitrary number of `write` calls.
    fn write(&mut self, bytes: &[u8]);
}

/// Trait for types that can be formatted via defmt.
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
/// Usually, an implementation of this trait can be `#[derive]`d automatically:
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
///
/// Manual implementations can make use of the [`write!`] macro:
///
/// ```
/// use defmt::{Format, Formatter, write};
///
/// struct Id(u32);
///
/// impl Format for Id {
///     fn format(&self, fmt: Formatter) {
///         // Format as hexadecimal.
///         write!(fmt, "Id({:x})", self.0);
///     }
/// }
/// ```
///
/// Note that [`write!`] can only be called once, as it consumes the [`Formatter`].
pub trait Format {
    /// Writes the defmt representation of `self` to `fmt`.
    fn format(&self, fmt: Formatter);
}

#[export_name = "__defmt_default_timestamp"]
fn default_timestamp(_f: Formatter<'_>) {
    // By default, no timestamp is used.
}

// There is no default timestamp format. Instead, the decoder looks for a matching ELF symbol. If
// absent, timestamps are turned off.

#[export_name = "__defmt_default_panic"]
fn default_panic() -> ! {
    core::panic!()
}

/// An "adapter" type to feed `Debug` values into defmt macros, which expect `defmt::Format` values.
///
/// This adapter disables compression and uses the `core::fmt` code on-device! You should prefer
/// `defmt::Format` over `Debug` whenever possible.
///
/// example usage:
///
/// ```rust
/// # #[derive(Debug)]
/// # struct ExpensiveThing();
/// # let expensive_thing = ExpensiveThing();
/// #
/// defmt::info!("{:?}", defmt::Debug2Format(&expensive_thing));
/// //                                        ˆˆˆˆˆˆˆˆˆˆˆˆˆˆˆ
/// //                                        must `#[derive(Debug)]`
/// ```
///
/// Note that any provided defmt display hints will be ignored
/// because this always uses `{:?}` to format the contained value.
pub struct Debug2Format<'a, T: fmt::Debug + ?Sized>(pub &'a T);

impl<T: fmt::Debug + ?Sized> Format for Debug2Format<'_, T> {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = defmt_macros::internp!("{=__internal_Debug}");
            fmt.inner.u8(&t);
        }
        fmt.inner.debug(&self.0);
    }
}

/// An "adapter" type to feed `Display` values into defmt macros, which expect `defmt::Format` values.
///
/// This adapter disables compression and uses the `core::fmt` code on-device! You should prefer
/// `defmt::Format` over `Display` whenever possible.
///
/// example usage:
///
/// ```rust
/// # struct ExpensiveThing();
///
/// # impl core::fmt::Display for ExpensiveThing {
/// #     fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
/// #         write!(f, "{}", "expensive")
/// #     }
/// #  }
/// # let expensive_thing = ExpensiveThing();
/// #
/// defmt::info!("{}", defmt::Display2Format(&expensive_thing));
/// //                                        ˆˆˆˆˆˆˆˆˆˆˆˆˆˆˆ
/// //                                        must implement `fmt::Display`
/// ```
///
/// Note that any provided defmt display hints will be ignored
/// because this always uses `{}` to format the contained value.
pub struct Display2Format<'a, T: fmt::Display + ?Sized>(pub &'a T);

impl<T: fmt::Display + ?Sized> Format for Display2Format<'_, T> {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = defmt_macros::internp!("{=__internal_Display}");
            fmt.inner.u8(&t);
        }
        fmt.inner.display(&self.0);
    }
}
