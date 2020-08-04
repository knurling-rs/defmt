use crate as binfmt;

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
fn str() {
    binfmt::info!("Hello, {:str}", "world");

    let world = binfmt::intern!("world");
    binfmt::info!("Hello, {:istr}", world);
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
