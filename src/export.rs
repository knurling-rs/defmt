use crate::{Formatter, Str};

pub use defmt_common::Level;
pub use defmt_macros::write;

#[cfg(target_arch = "x86_64")]
thread_local! {
    static I: core::sync::atomic::AtomicU8 =
        core::sync::atomic::AtomicU8::new(0);
    static T: core::sync::atomic::AtomicU8 =
        core::sync::atomic::AtomicU8::new(0);
}

// NOTE we limit these values to 7-bit to avoid LEB128 encoding while writing the expected answers
// in unit tests
/// For testing purposes
#[cfg(target_arch = "x86_64")]
pub fn fetch_string_index() -> u8 {
    I.with(|i| i.load(core::sync::atomic::Ordering::Relaxed)) & 0x7f
}

/// For testing purposes
#[cfg(target_arch = "x86_64")]
pub fn fetch_add_string_index() -> usize {
    (I.with(|i| i.fetch_add(1, core::sync::atomic::Ordering::Relaxed)) & 0x7f) as usize
}

/// For testing purposes
#[cfg(target_arch = "x86_64")]
pub fn fetch_timestamp() -> u8 {
    T.with(|i| i.load(core::sync::atomic::Ordering::Relaxed)) & 0x7f
}

pub fn threshold() -> Level {
    // TODO add Cargo features
    Level::Info
}

#[cfg(target_arch = "x86_64")]
pub fn acquire() -> Option<Formatter> {
    None
}

#[cfg(not(target_arch = "x86_64"))]
pub fn acquire() -> Option<Formatter> {
    extern "Rust" {
        fn _defmt_acquire() -> Option<Formatter>;
    }
    unsafe { _defmt_acquire() }
}

#[cfg(target_arch = "x86_64")]
pub fn release(_: Formatter) {}

#[cfg(not(target_arch = "x86_64"))]
pub fn release(fmt: Formatter) {
    extern "Rust" {
        fn _defmt_release(fmt: Formatter);
    }
    unsafe { _defmt_release(fmt) }
}

#[cfg(target_arch = "x86_64")]
pub fn timestamp() -> u64 {
    (T.with(|i| i.fetch_add(1, core::sync::atomic::Ordering::Relaxed)) & 0x7f) as u64
}

/// For testing purposes
#[cfg(not(target_arch = "x86_64"))]
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
}

pub fn truncate<T>(x: impl sealed::Truncate<T>) -> T {
    x.truncate()
}
