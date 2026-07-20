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
    defmt::info!("Hello, {=str}", "world");

    let world = defmt::intern!("world");
    defmt::info!("Hello, {=istr}", world);
}

#[test]
fn named_arguments() {
    let owner = 42u8;
    let count = 7u16;

    // inline capture from the surrounding scope
    defmt::info!("{owner=u8}");
    defmt::info!("{owner}");
    defmt::info!("{owner:?}"); // std's Debug syntax works unchanged
    defmt::info!("{owner=u8:#x} {count=u16}");

    // repeated use of one capture serializes the argument only once
    defmt::info!("{owner} and {owner}");

    // explicitly supplied named arguments
    defmt::info!("{x=u8}", x = 1);
    defmt::info!("{x}", x = owner);

    // mixed positional and named
    defmt::info!("{=u8} {owner=u8} {0=u8}", 1);
}

#[test]
fn named_arguments_in_write() {
    struct S {
        x: u8,
    }

    impl defmt::Format for S {
        fn format(&self, f: defmt::Formatter) {
            let x = self.x;
            defmt::write!(f, "S {{ x: {x=u8} }}")
        }
    }

    defmt::info!("{}", S { x: 1 });
}

#[test]
fn named_arguments_in_assert_like() {
    let owner = 42u8;
    defmt::assert!(true, "{owner}");
    defmt::assert_eq!(1, 1, "{owner=u8:#x}");
}

#[test]
fn trailing_comma() {
    defmt::trace!("test trace",);
    defmt::debug!("test debug",);
    defmt::info!("test info",);
    defmt::warn!("test warn",);
    defmt::error!("test error",);

    defmt::trace!("test trace {=?}", 0,);
    defmt::debug!("test debug {=?}", 0,);
    defmt::info!("test info {=?}", 0,);
    defmt::warn!("test warn {=?}", 0,);
    defmt::error!("test error {=?}", 0,);
}
