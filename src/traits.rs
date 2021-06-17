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
/// The global logger can be acquired once for each "execution context". The definition
/// of execution context is up to the implementation. For example, it can be:
///
/// - the entire process.
/// - one thread in std environments.
/// - one interrupt priority level in embedded devices.
///
/// # Safety contract
///
/// - `acquire` logically acquires the global logger in the current execution context.
///   The acquiring is tracked internally, no Rust object is returned representing ownership.
/// - `acquire` is a safe function, therefore it must be thread-safe and interrupt-safe
///
/// And, not safety related, the methods should never be invoked from user code. The easiest way to
/// ensure this is to implement `Logger` on a *private* `struct` and mark that `struct` as the
/// `#[global_logger]`.
pub unsafe trait Logger {
    /// Acquire the global logger in the current execution context.
    ///
    /// Panics if already acquired in the current execution context. Otherwise it must never fail.
    fn acquire();

    /// Releases the global logger in the current execution context.
    ///
    /// # Safety
    /// Must be called exactly once for each acquire(), in the same execution context.
    unsafe fn release();

    /// Writes `bytes` to the destination.
    ///
    /// This will be called by the defmt logging macros to transmit encoded data. The write
    /// operation must not fail.
    ///
    /// Note that a call to `write` does *not* correspond to a defmt logging macro invocation. A
    /// single `defmt::info!` call can result in an arbitrary number of `write` calls.
    ///
    /// # Safety
    /// Must only be called when the global logger is acquired in the current execution context.
    /// (i.e. between acquire() and release())
    unsafe fn write(bytes: &[u8]);
}
