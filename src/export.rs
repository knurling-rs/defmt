use crate::{InternalFormatter, Str};

#[cfg(feature = "unstable-test")]
thread_local! {
    static I: core::sync::atomic::AtomicU8 =
        core::sync::atomic::AtomicU8::new(0);
    static T: core::sync::atomic::AtomicU8 =
        core::sync::atomic::AtomicU8::new(0);
}

// NOTE we limit these values to 7-bit to avoid LEB128 encoding while writing the expected answers
// in unit tests
/// For testing purposes
#[cfg(feature = "unstable-test")]
pub fn fetch_string_index() -> u8 {
    I.with(|i| i.load(core::sync::atomic::Ordering::Relaxed)) & 0x7f
}

/// For testing purposes
#[cfg(feature = "unstable-test")]
pub fn fetch_add_string_index() -> usize {
    (I.with(|i| i.fetch_add(1, core::sync::atomic::Ordering::Relaxed)) & 0x7f) as usize
}

#[cfg(feature = "unstable-test")]
pub fn acquire() -> Option<InternalFormatter> {
    None
}

#[cfg(not(feature = "unstable-test"))]
#[inline(never)]
pub fn acquire() -> Option<InternalFormatter> {
    extern "Rust" {
        fn _defmt_acquire() -> Option<InternalFormatter>;
    }
    unsafe { _defmt_acquire() }
}

#[cfg(feature = "unstable-test")]
pub fn release(_: InternalFormatter) {}

#[cfg(not(feature = "unstable-test"))]
#[inline(never)]
pub fn release(fmt: InternalFormatter) {
    extern "Rust" {
        fn _defmt_release(fmt: InternalFormatter);
    }
    unsafe { _defmt_release(fmt) }
}

/// For testing purposes
#[cfg(feature = "unstable-test")]
pub fn timestamp() -> u64 {
    (T.with(|i| i.fetch_add(1, core::sync::atomic::Ordering::Relaxed)) & 0x7f) as u64
}

#[cfg(not(feature = "unstable-test"))]
pub fn timestamp() -> u64 {
    extern "Rust" {
        fn _defmt_timestamp() -> u64;
    }
    unsafe { _defmt_timestamp() }
}

/// Returns the interned string at `address`.
pub fn istr(address: usize) -> Str {
    Str {
        // NOTE address is limited to 14 bits in the linker script
        address: address as *const u8 as u16,
    }
}

mod sealed {
    #[allow(unused_imports)]
    use crate as defmt;
    use crate::{Format, Formatter};
    use defmt_macros::internp;

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

    // needed so we can call truncate() without having to check whether truncation is necessary first
    impl Truncate<u64> for u64 {
        fn truncate(self) -> u64 {
            self
        }
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct NoneError;

    impl Format for NoneError {
        fn format(&self, fmt: Formatter) {
            if fmt.inner.needs_tag() {
                let t = internp!("Unwrap of a None option value");
                fmt.inner.u8(&t);
            }
        }
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
