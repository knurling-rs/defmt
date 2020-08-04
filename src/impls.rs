#[cfg(target_arch = "x86_64")]
use crate as binfmt;
use binfmt_macros::internp;

use crate::{Format, Formatter, Str};

impl Format for i8 {
    fn format(&self, fmt: &mut Formatter) {
        let t = internp!("{:i8}");
        fmt.write(&[t, *self as u8]);
    }
}

impl Format for i16 {
    fn format(&self, fmt: &mut Formatter) {
        let t = internp!("{:i16}");
        let x = *self as u16;
        fmt.write(&[t, x as u8, (x >> 8) as u8]);
    }
}

impl Format for i32 {
    fn format(&self, fmt: &mut Formatter) {
        let t = internp!("{:i32}");
        fmt.u8(&t);
        fmt.i32(self);
    }
}

impl Format for u8 {
    fn format(&self, fmt: &mut Formatter) {
        let t = internp!("{:u8}");
        fmt.write(&[t, *self]);
    }
}

impl Format for u16 {
    fn format(&self, fmt: &mut Formatter) {
        let t = internp!("{:u16}");
        fmt.u8(&t);
        fmt.u16(self);
    }
}

impl Format for u32 {
    fn format(&self, fmt: &mut Formatter) {
        let t = internp!("{:u32}");
        fmt.u8(&t);
        fmt.u32(self);
    }
}

impl Format for Str {
    fn format(&self, fmt: &mut Formatter) {
        let t = internp!("{:str}");
        fmt.u8(&t);
        fmt.istr(self);
    }
}

impl<T> Format for Option<T>
where
    T: Format,
{
    fn format(&self, f: &mut Formatter) {
        let t = internp!("None|Some({})");
        match self {
            None => f.write(&[t, 0]),
            Some(x) => {
                f.write(&[t, 1]);
                x.format(f);
            }
        }
    }
}

impl<T, E> Format for Result<T, E>
where
    T: Format,
    E: Format,
{
    fn format(&self, f: &mut Formatter) {
        let t = internp!("Err({:?})|Ok({:?})");
        match self {
            Err(e) => {
                f.write(&[t, 0]);
                e.format(f)
            }
            Ok(x) => {
                f.write(&[t, 1]);
                x.format(f);
            }
        }
    }
}
