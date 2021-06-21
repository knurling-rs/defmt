use super::*;

impl Format for i8 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=i8}");
            fmt.inner.tag(&t);
        }
        fmt.inner.u8(&(*self as u8));
    }
}

impl Format for i16 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=i16}");
            fmt.inner.tag(&t);
        }
        fmt.inner.u16(&(*self as u16))
    }
}

impl Format for i32 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=i32}");
            fmt.inner.tag(&t);
        }
        fmt.inner.i32(self);
    }
}

impl Format for i64 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=i64}");
            fmt.inner.tag(&t);
        }
        fmt.inner.i64(self);
    }
}

impl Format for i128 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=i128}");
            fmt.inner.tag(&t);
        }
        fmt.inner.i128(self);
    }
}

impl Format for isize {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=isize}");
            fmt.inner.tag(&t);
        }
        fmt.inner.isize(self);
    }
}

impl Format for u8 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=u8}");
            fmt.inner.tag(&t);
        }
        fmt.inner.u8(self)
    }
}

impl Format for u16 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=u16}");
            fmt.inner.tag(&t);
        }
        fmt.inner.u16(self);
    }
}

impl Format for u32 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=u32}");
            fmt.inner.tag(&t);
        }
        fmt.inner.u32(self);
    }
}

impl Format for u64 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=u64}");
            fmt.inner.tag(&t);
        }
        fmt.inner.u64(self);
    }
}

impl Format for u128 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=u128}");
            fmt.inner.tag(&t);
        }
        fmt.inner.u128(self);
    }
}

impl Format for usize {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=usize}");
            fmt.inner.tag(&t);
        }
        fmt.inner.usize(self);
    }
}

impl Format for f32 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=f32}");
            fmt.inner.tag(&t);
        }
        fmt.inner.f32(self);
    }
}

impl Format for f64 {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=f64}");
            fmt.inner.tag(&t);
        }
        fmt.inner.f64(self);
    }
}

impl Format for str {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = str_tag();
            fmt.inner.tag(&t);
        }
        fmt.inner.str(self);
    }
}

pub(crate) fn str_tag() -> u16 {
    internp!("{=str}")
}

impl Format for Str {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=istr}");
            fmt.inner.tag(&t);
        }
        fmt.inner.istr(self);
    }
}

impl Format for char {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=char}");
            fmt.inner.tag(&t);
        }
        fmt.inner.u32(&(*self as u32));
    }
}

impl<T> Format for [T]
where
    T: Format,
{
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=[?]}");
            fmt.inner.tag(&t);
        }
        fmt.inner.fmt_slice(self)
    }
}

impl<T> Format for &'_ T
where
    T: Format + ?Sized,
{
    fn format(&self, fmt: Formatter) {
        T::format(self, fmt)
    }
}

impl<T> Format for &'_ mut T
where
    T: Format + ?Sized,
{
    fn format(&self, fmt: Formatter) {
        T::format(self, fmt)
    }
}

impl Format for bool {
    fn format(&self, fmt: Formatter) {
        if fmt.inner.needs_tag() {
            let t = internp!("{=bool}");
            fmt.inner.tag(&t);
        }
        fmt.inner.bool(self);
    }
}

impl Format for () {
    fn format(&self, f: Formatter) {
        if f.inner.needs_tag() {
            let t = internp!("()");
            f.inner.tag(&t);
        }
    }
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
