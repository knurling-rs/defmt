#![allow(warnings)] // FIXME
#![cfg_attr(not(target_arch = "x86_64"), no_std)]

use core::{mem::MaybeUninit, ptr::NonNull};

#[cfg(not(test))]
pub use binfmt_macros::intern;
#[doc(hidden)]
pub use binfmt_macros::winfo;
pub use binfmt_macros::{info, write, Format};

use crate as binfmt;

#[cfg(test)]
macro_rules! intern {
    ($s:expr) => {
        crate::Str {
            address: crate::tests::STR,
        }
    };
}

#[doc(hidden)]
pub mod export;
mod impls;
#[cfg(test)]
mod tests;

/// Interned string
#[derive(Clone, Copy)]
pub struct Str {
    // 14-bit address
    address: u16,
}

/// Handler that owns the global logger
pub struct Formatter {
    #[cfg(not(target_arch = "x86_64"))]
    writer: NonNull<dyn Write>,
    #[cfg(target_arch = "x86_64")]
    bytes: Vec<u8>,
}

/// # Unsafety
/// `buf` must be large enough to hold the encoded value
unsafe fn leb64(x: u64, buf: &mut [u8]) -> usize {
    let mut low = x as u32;
    let mut high = (x >> 32) as u32;

    let mut i = 0;
    loop {
        let mut byte = (low & 0x7f) as u8;
        low >>= 7;
        if low != 0 {
            byte |= 0x80;
        }

        *buf.get_unchecked_mut(i) = byte;
        i += 1;
        if low == 0 {
            break;
        }
    }

    if high == 0 {
        return i;
    }

    for j in (i - 1)..4 {
        *buf.get_unchecked_mut(j) = 0x80;
    }

    if i != 5 {
        *buf.get_unchecked_mut(4) = 0;
    }

    i = 4;
    *buf.get_unchecked_mut(i) |= (high as u8 & 0b111) << 4;
    high >>= 3;

    if high != 0 {
        *buf.get_unchecked_mut(i) |= 0x80;
    }

    i += 1;

    if high == 0 {
        return i;
    }

    loop {
        let mut byte = (high & 0x7f) as u8;
        high >>= 7;
        if high != 0 {
            byte |= 0x80;
        }

        *buf.get_unchecked_mut(i) = byte;
        i += 1;
        if high == 0 {
            return i;
        }
    }
}

impl Formatter {
    /// Only for testing
    #[cfg(target_arch = "x86_64")]
    pub fn new() -> Self {
        Self { bytes: vec![] }
    }

    /// Only for testing
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

    #[cfg(not(target_arch = "x86_64"))]
    pub unsafe fn from_raw(writer: *mut dyn Write) -> Self {
        Self {
            writer: NonNull::new_unchecked(writer),
        }
    }

    // TODO turn these public methods in `export` free functions
    pub fn fmt(&mut self, f: &impl Format) {
        f.format(self)
    }

    pub fn leb64(&mut self, x: u64) {
        let mut buf: [u8; 10] = unsafe { MaybeUninit::uninit().assume_init() };
        let i = unsafe { leb64(x, &mut buf) };
        self.write(unsafe { buf.get_unchecked(..i) })
    }

    pub fn i8(&mut self, b: &i8) {
        self.write(&b.to_le_bytes())
    }

    pub fn i16(&mut self, b: &i16) {
        self.write(&b.to_le_bytes())
    }

    pub fn i32(&mut self, b: &i32) {
        self.write(&b.to_le_bytes())
    }

    // TODO remove
    pub fn prim(&mut self, s: &Str) {
        self.write(&[s.address as u8])
    }

    pub fn u8(&mut self, b: &u8) {
        self.write(&[*b])
    }

    pub fn u16(&mut self, b: &u16) {
        self.write(&b.to_le_bytes())
    }

    pub fn u24(&mut self, b: &u32) {
        self.write(&b.to_le_bytes()[..3])
    }

    pub fn u32(&mut self, b: &u32) {
        self.write(&b.to_le_bytes())
    }

    pub fn str(&mut self, s: &Str) {
        // LEB128 encoding
        if s.address < 128 {
            self.write(&[s.address as u8])
        } else {
            self.write(&[s.address as u8 | (1 << 7), (s.address >> 7) as u8])
        }
    }
}

pub trait Write {
    fn write(&mut self, bytes: &[u8]);
}

pub trait Format {
    fn format(&self, fmt: &mut Formatter);
}
