//! Some of these objects don't expose enough to accurately report their debug state. In this case
//! we show as much state as we can. Users can always use `Debug2Format` to get more information,
//! at the cost of bringing core::fmt into the firmware and doing the layout work on device.
//!
//! We generally keep the type parameter trait bounds in case it becomes possible to use this
//! later, without making a backwards-incompatible change.

mod num;
mod ops;
mod slice;

use super::*;

impl<T> Format for Option<T>
where
    T: Format,
{
    fn format(&self, f: Formatter) {
        if f.inner.needs_tag() {
            let t = internp!("None|Some({=?})");
            f.inner.u8(&t);
        }
        match self {
            None => f.inner.u8(&0),
            Some(x) => {
                f.inner.u8(&1);
                f.inner.with_tag(|f| x.format(f))
            }
        }
    }
}

impl<T, E> Format for Result<T, E>
where
    T: Format,
    E: Format,
{
    fn format(&self, f: Formatter) {
        if f.inner.needs_tag() {
            let t = internp!("Err({=?})|Ok({=?})");
            f.inner.u8(&t);
        }
        match self {
            Err(e) => {
                f.inner.u8(&0);
                f.inner.with_tag(|f| e.format(f))
            }
            Ok(x) => {
                f.inner.u8(&1);
                f.inner.with_tag(|f| x.format(f))
            }
        }
    }
}

impl<T> Format for core::marker::PhantomData<T> {
    fn format(&self, f: Formatter) {
        if f.inner.needs_tag() {
            let t = internp!("PhantomData");
            f.inner.u8(&t);
        }
    }
}

impl Format for core::convert::Infallible {
    #[inline]
    fn format(&self, _: Formatter) {
        // type cannot be instantiated so nothing to do here
        match *self {}
    }
}

impl Format for core::time::Duration {
    fn format(&self, fmt: Formatter) {
        crate::write!(
            fmt,
            "Duration {{ secs: {=u64}, nanos: {=u32} }}",
            self.as_secs(),
            self.subsec_nanos(),
        )
    }
}

impl<A, B> Format for core::iter::Zip<A, B>
where
    A: Format,
    B: Format,
{
    fn format(&self, fmt: Formatter) {
        crate::write!(fmt, "Zip(..)")
    }
}
