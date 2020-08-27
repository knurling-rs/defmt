use crate as defmt;

#[test]
fn log_levels() {
    // just make sure they build OK for now
    defmt::trace!("test trace");
    defmt::debug!("test debug");
    defmt::info!("test info");
    defmt::warn!("test warn");
    defmt::error!("test error");
}

#[test]
fn str() {
    defmt::info!("Hello, {:str}", "world");

    let world = defmt::intern!("world");
    defmt::info!("Hello, {:istr}", world);

    defmt::info!("Hello, {:bstr}", &b"world"[..]);
}

#[test]
fn trailing_comma() {
    defmt::trace!("test trace",);
    defmt::debug!("test debug",);
    defmt::info!("test info",);
    defmt::warn!("test warn",);
    defmt::error!("test error",);

    defmt::trace!("test trace {:?}", 0,);
    defmt::debug!("test debug {:?}", 0,);
    defmt::info!("test info {:?}", 0,);
    defmt::warn!("test warn {:?}", 0,);
    defmt::error!("test error {:?}", 0,);

    // Don't run this code, just check that it builds.
    #[allow(unreachable_code, unused_variables)]
    if false {
        let fmt: defmt::Formatter = panic!();
        defmt::export::write!(fmt, "test write",);
        defmt::export::write!(fmt, "test write {:?}", 0,);
    }
}
