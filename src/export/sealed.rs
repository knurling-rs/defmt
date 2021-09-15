#[allow(unused_imports)]
use crate as defmt;
use crate::{Format, Formatter, Str};

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

impl Truncate<u8> for u128 {
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

impl Truncate<u16> for u128 {
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

impl Truncate<u32> for u128 {
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

impl Truncate<u64> for u128 {
    fn truncate(self) -> u64 {
        self as u64
    }
}

// needed so we can call truncate() without having to check whether truncation is necessary first
impl Truncate<u128> for u128 {
    fn truncate(self) -> u128 {
        self
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct NoneError;

impl Format for NoneError {
    fn format(&self, _fmt: Formatter) {
        unreachable!();
    }

    fn _format_tag() -> Str {
        defmt_macros::internp!("Unwrap of a None option value")
    }

    fn _format_data(&self) {}
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
