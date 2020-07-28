use crate::{Formatter, Str};

pub use common::Level;

#[cfg(target_arch = "x86_64")]
thread_local! {
    static I: core::sync::atomic::AtomicUsize =
        core::sync::atomic::AtomicUsize::new(0);
    static T: core::sync::atomic::AtomicU64 =
        core::sync::atomic::AtomicU64::new(0);
}

#[cfg(target_arch = "x86_64")]
pub fn fetch_string_index() -> usize {
    I.with(|i| i.load(core::sync::atomic::Ordering::Relaxed))
}

#[cfg(target_arch = "x86_64")]
pub fn fetch_add_string_index() -> usize {
    I.with(|i| i.fetch_add(1, core::sync::atomic::Ordering::Relaxed))
}

#[cfg(target_arch = "x86_64")]
pub fn fetch_timestamp() -> u64 {
    T.with(|i| i.load(core::sync::atomic::Ordering::Relaxed))
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
        fn _binfmt_acquire() -> Option<Formatter>;
    }
    unsafe { _binfmt_acquire() }
}

#[cfg(target_arch = "x86_64")]
pub fn release(_: Formatter) {}

#[cfg(not(target_arch = "x86_64"))]
pub fn release(fmt: Formatter) {
    extern "Rust" {
        fn _binfmt_release(fmt: Formatter);
    }
    unsafe { _binfmt_release(fmt) }
}

#[cfg(target_arch = "x86_64")]
pub fn timestamp() -> u64 {
    T.with(|i| i.fetch_add(1, core::sync::atomic::Ordering::Relaxed))
}

#[cfg(not(target_arch = "x86_64"))]
pub fn timestamp() -> u64 {
    extern "Rust" {
        fn _binfmt_timestamp() -> u64;
    }
    unsafe { _binfmt_timestamp() }
}

pub fn str(address: usize) -> Str {
    Str {
        // NOTE address is limited to 14 bits in the linker script
        #[cfg(not(test))]
        address: address as *const u8 as u16,
        #[cfg(test)]
        address: crate::tests::STR,
    }
}
