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

impl Format for isize {
    fn format(&self, fmt: &mut Formatter) {
        let t = internp!("{:isize}");
        fmt.u8(&t);
        fmt.isize(self);
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

impl Format for usize {
    fn format(&self, fmt: &mut Formatter) {
        let t = internp!("{:usize}");
        fmt.u8(&t);
        fmt.usize(self);
    }
}

impl Format for f32 {
    fn format(&self, fmt: &mut Formatter) {
        // to_bits then to_le_bytes
        let t = internp!("{:f32}");
        fmt.u8(&t);
        fmt.f32(self);
    }
}

impl Format for Str {
    fn format(&self, fmt: &mut Formatter) {
        let t = internp!("{:str}");
        fmt.u8(&t);
        fmt.istr(self);
    }
}

impl Format for &[u8] {
    fn format(&self, fmt: &mut Formatter) {
        let t = internp!("{:[u8]}");
        fmt.u8(&t);
        fmt.slice(self);
    }
}

macro_rules! arrays {
    ( $($len:literal $fmt:literal,)+ ) => { $(
        impl Format for [u8; $len] {
            fn format(&self, fmt: &mut Formatter) {
                let t = internp!($fmt);
                fmt.u8(&t);
                fmt.array(self);
            }
        }
    )+ };
}

arrays! {
    0 "{:[u8; 0]}",
    1 "{:[u8; 1]}",
    2 "{:[u8; 2]}",
    3 "{:[u8; 3]}",
    4 "{:[u8; 4]}",
    5 "{:[u8; 5]}",
    6 "{:[u8; 6]}",
    7 "{:[u8; 7]}",
    8 "{:[u8; 8]}",
    9 "{:[u8; 9]}",
    10 "{:[u8; 10]}",
    11 "{:[u8; 11]}",
    12 "{:[u8; 12]}",
    13 "{:[u8; 13]}",
    14 "{:[u8; 14]}",
    15 "{:[u8; 15]}",
    16 "{:[u8; 16]}",
    17 "{:[u8; 17]}",
    18 "{:[u8; 18]}",
    19 "{:[u8; 19]}",
    20 "{:[u8; 20]}",
    21 "{:[u8; 21]}",
    22 "{:[u8; 22]}",
    23 "{:[u8; 23]}",
    24 "{:[u8; 24]}",
    25 "{:[u8; 25]}",
    26 "{:[u8; 26]}",
    27 "{:[u8; 27]}",
    28 "{:[u8; 28]}",
    29 "{:[u8; 29]}",
    30 "{:[u8; 30]}",
    31 "{:[u8; 31]}",
    32 "{:[u8; 32]}",
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
