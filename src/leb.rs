pub(crate) fn leb64(x: u64, buf: &mut [u8; 10]) -> usize {
    let mut low = x as u32;
    let mut high = (x >> 32) as u32;

    let mut i = 0;
    loop {
        let mut byte = (low & 0x7f) as u8;
        low >>= 7;
        if low != 0 {
            byte |= 0x80;
        }

        buf[i] = byte;
        i += 1;
        if low == 0 {
            break;
        }
    }

    if high == 0 {
        return i;
    }

    for j in (i - 1)..4 {
        buf[j] = 0x80;
    }

    if i != 5 {
        buf[4] = 0;
    }

    i = 4;
    buf[i] |= (high as u8 & 0b111) << 4;
    high >>= 3;

    if high != 0 {
        buf[i] |= 0x80;
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

        buf[i] = byte;
        i += 1;
        if high == 0 {
            return i;
        }
    }
}

pub fn zigzag_encode(v: i64) -> u64 {
    ((v << 1) ^ (v >> 63)) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leb() {
        let mut buf = [0x55; 10];

        let i = leb64(0, &mut buf);
        assert_eq!(buf[..i], [0]);
        buf.iter_mut().for_each(|b| *b = 0x55);

        let i = leb64(1, &mut buf);
        assert_eq!(buf[..i], [1]);
        buf.iter_mut().for_each(|b| *b = 0x55);

        let i = leb64((1 << 7) - 1, &mut buf);
        assert_eq!(buf[..i], [0x7f]);
        buf.iter_mut().for_each(|b| *b = 0x55);

        let i = leb64(1 << 7, &mut buf);
        assert_eq!(buf[..i], [0x80, 1]);
        buf.iter_mut().for_each(|b| *b = 0x55);

        let i = leb64((1 << 32) - 1, &mut buf);
        assert_eq!(buf[..i], [0xff, 0xff, 0xff, 0xff, 0xf]);
        buf.iter_mut().for_each(|b| *b = 0x55);

        let i = leb64((1 << 35) - 1, &mut buf);
        assert_eq!(buf[..i], [0xff, 0xff, 0xff, 0xff, 0x7f]);
        buf.iter_mut().for_each(|b| *b = 0x55);

        let i = leb64(1 << 35, &mut buf);
        assert_eq!(buf[..i], [0x80, 0x80, 0x80, 0x80, 0x80, 1]);
        buf.iter_mut().for_each(|b| *b = 0x55);

        let i = leb64((1 << 42) - 1, &mut buf);
        assert_eq!(buf[..i], [0xff, 0xff, 0xff, 0xff, 0xff, 0x7f]);
        buf.iter_mut().for_each(|b| *b = 0x55);

        let i = leb64(u64::max_value(), &mut buf);
        assert_eq!(
            buf[..i],
            [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 1]
        );
    }
}
