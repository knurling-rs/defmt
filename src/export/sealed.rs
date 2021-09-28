#[allow(unused_imports)]
use crate as defmt;
use crate::{Format, Formatter, Str};

pub trait Truncate<U> {
    fn truncate(self) -> U;
}

macro_rules! impl_truncate {
    ($t:ty => $u:ty) => {
        impl Truncate<$t> for $u {
            fn truncate(self) -> $t {
                self as $t
            }
        }
    };
}

// We implement Truncate<X> for X so that the macro can unconditionally use it,
// even if no truncation is performed.

impl_truncate!(u8 => u8);
impl_truncate!(u8 => u16);
impl_truncate!(u8 => u32);
impl_truncate!(u8 => u64);
impl_truncate!(u8 => u128);
impl_truncate!(u16 => u16);
impl_truncate!(u16 => u32);
impl_truncate!(u16 => u64);
impl_truncate!(u16 => u128);
impl_truncate!(u32 => u32);
impl_truncate!(u32 => u64);
impl_truncate!(u32 => u128);
impl_truncate!(u64 => u64);
impl_truncate!(u64 => u128);
impl_truncate!(u128 => u128);

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
