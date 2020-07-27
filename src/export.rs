use crate::{Formatter, Str};

pub use common::Level;

pub fn threshold() -> Level {
    // TODO add Cargo features
    Level::Info
}

pub fn acquire() -> Option<Formatter> {
    extern "Rust" {
        fn _binfmt_acquire() -> Option<Formatter>;
    }
    unsafe { _binfmt_acquire() }
}

pub fn release(fmt: Formatter) {
    extern "Rust" {
        fn _binfmt_release(fmt: Formatter);
    }
    unsafe { _binfmt_release(fmt) }
}

pub fn timestamp() -> u64 {
    extern "Rust" {
        fn _binfmt_timestamp() -> u64;
    }
    unsafe { _binfmt_timestamp() }
}

pub fn str(address: &'static u8) -> Str {
    Str {
        // NOTE address is limited to 14 bits in the linker script
        #[cfg(not(test))]
        address: address as *const u8 as u16,
        #[cfg(test)]
        address: crate::tests::STR,
    }
}
