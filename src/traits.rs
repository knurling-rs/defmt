use core::ptr::NonNull;

use crate::Formatter;

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
