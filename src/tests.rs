use crate::{self as binfmt, Format, Formatter};

impl super::Write for Vec<u8> {
    fn write(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes)
    }
}

pub static PSTR: u8 = 0x7F;
pub static STR: u16 = 0x3FFF;
static STRB: [u8; 2] = [(STR & 0x7f) as u8 | (1 << 7), (STR >> 7) as u8];

fn check(prim: impl Format, bytes: &[u8]) {
    let mut f = Formatter::new();
    prim.format(&mut f);
    assert_eq!(f.bytes, bytes);
}

#[test]
fn format() {
    check(42u8, &[PSTR, 42]);

    check(42u16, &[PSTR, 42, 0]);
    check(513u16, &[PSTR, 1, 2]);

    check(42u32, &[PSTR, 42, 0, 0, 0]);
    check(513u32, &[PSTR, 1, 2, 0, 0]);

    check(42i8, &[PSTR, 42]);
    check(-42i8, &[PSTR, -42i8 as u8]);

    check(None::<u8>, &[PSTR, 0]);
    check(Some(42u8), &[PSTR, 1, PSTR, 42]);
}

#[test]
fn leb() {
    let mut buf = [0x55; 10];

    let i = unsafe { super::leb64(0, &mut buf) };
    assert_eq!(buf[..i], [0]);
    buf.iter_mut().for_each(|b| *b = 0x55);

    let i = unsafe { super::leb64(1, &mut buf) };
    assert_eq!(buf[..i], [1]);
    buf.iter_mut().for_each(|b| *b = 0x55);

    let i = unsafe { super::leb64((1 << 7) - 1, &mut buf) };
    assert_eq!(buf[..i], [0x7f]);
    buf.iter_mut().for_each(|b| *b = 0x55);

    let i = unsafe { super::leb64(1 << 7, &mut buf) };
    assert_eq!(buf[..i], [0x80, 1]);
    buf.iter_mut().for_each(|b| *b = 0x55);

    let i = unsafe { super::leb64((1 << 32) - 1, &mut buf) };
    assert_eq!(buf[..i], [0xff, 0xff, 0xff, 0xff, 0xf]);
    buf.iter_mut().for_each(|b| *b = 0x55);

    let i = unsafe { super::leb64((1 << 35) - 1, &mut buf) };
    assert_eq!(buf[..i], [0xff, 0xff, 0xff, 0xff, 0x7f]);
    buf.iter_mut().for_each(|b| *b = 0x55);

    let i = unsafe { super::leb64(1 << 35, &mut buf) };
    assert_eq!(buf[..i], [0x80, 0x80, 0x80, 0x80, 0x80, 1]);
    buf.iter_mut().for_each(|b| *b = 0x55);

    let i = unsafe { super::leb64((1 << 42) - 1, &mut buf) };
    assert_eq!(buf[..i], [0xff, 0xff, 0xff, 0xff, 0xff, 0x7f]);
    buf.iter_mut().for_each(|b| *b = 0x55);

    let i = unsafe { super::leb64(u64::max_value(), &mut buf) };
    assert_eq!(
        buf[..i],
        [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 1]
    );
}

#[test]
fn derive() {
    #[derive(Format)]
    struct X {
        y: u8,
        z: u16,
    }

    let x = X { y: 1, z: 2 };
    check(x, &[STRB[0], STRB[1], 1, 2, 0]);
}
