use super::*;

macro_rules! prim {
    ($ty:ty, $method:ident, $fmt: literal) => {
        impl Format for $ty {
            default_format!();

            #[inline]
            fn _format_tag() -> u16 {
                internp!($fmt)
            }

            #[inline]
            fn _format_data(&self, fmt: Formatter) {
                fmt.inner.$method(self);
            }
        }
    };
}

prim!(i8, i8, "{=i8}");
prim!(i16, i16, "{=i16}");
prim!(i32, i32, "{=i32}");
prim!(i64, i64, "{=i64}");
prim!(i128, i128, "{=i128}");
prim!(isize, isize, "{=isize}");
prim!(u8, u8, "{=u8}");
prim!(u16, u16, "{=u16}");
prim!(u32, u32, "{=u32}");
prim!(u64, u64, "{=u64}");
prim!(u128, u128, "{=u128}");
prim!(usize, usize, "{=usize}");
prim!(f32, f32, "{=f32}");
prim!(f64, f64, "{=f64}");
prim!(str, str, "{=str}");
prim!(bool, bool, "{=bool}");
prim!(Str, istr, "{=istr}");

impl Format for char {
    default_format!();

    #[inline]
    fn _format_tag() -> u16 {
        internp!("{=char}")
    }

    #[inline]
    fn _format_data(&self, fmt: Formatter) {
        fmt.inner.u32(&(*self as u32));
    }
}

impl<T> Format for [T]
where
    T: Format,
{
    default_format!();

    #[inline]
    fn _format_tag() -> u16 {
        internp!("{=[?]}")
    }

    #[inline]
    fn _format_data(&self, fmt: Formatter) {
        fmt.inner.usize(&self.len());
        for value in self {
            fmt.inner.tag(T::_format_tag());
            value._format_data(Formatter { inner: fmt.inner });
        }
    }
}

impl<T> Format for &'_ T
where
    T: Format + ?Sized,
{
    delegate_format!(T, self, self);
}

impl<T> Format for &'_ mut T
where
    T: Format + ?Sized,
{
    delegate_format!(T, self, self);
}

// Format raw pointer as hexadecimal
//
// First cast raw pointer to thin pointer, then to usize and finally format as hexadecimal.
impl<T> Format for *const T
where
    T: ?Sized,
{
    fn format(&self, fmt: Formatter) {
        crate::write!(fmt, "{:x}", *self as *const () as usize);
    }
}

impl<T> Format for *mut T
where
    T: ?Sized,
{
    fn format(&self, fmt: Formatter) {
        Format::format(&(*self as *const T), fmt)
    }
}
