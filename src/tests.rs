use crate as binfmt;

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
fn log_levels() {
    // just make sure they build OK for now
    binfmt::trace!("test trace");
    binfmt::debug!("test debug");
    binfmt::info!("test info");
    binfmt::warn!("test warn");
    binfmt::error!("test error");
}

#[test]
fn trailing_comma() {
    binfmt::trace!("test trace",);
    binfmt::debug!("test debug",);
    binfmt::info!("test info",);
    binfmt::warn!("test warn",);
    binfmt::error!("test error",);

    binfmt::trace!("test trace {:?}", 0,);
    binfmt::debug!("test debug {:?}", 0,);
    binfmt::info!("test info {:?}", 0,);
    binfmt::warn!("test warn {:?}", 0,);
    binfmt::error!("test error {:?}", 0,);

    // Don't run this code, just check that it builds.
    #[allow(unreachable_code, unused_variables)]
    if false {
        let fmt: binfmt::Formatter = panic!();
        binfmt::write!(fmt, "test write",);
        binfmt::write!(fmt, "test write {:?}", 0,);
    }
}
