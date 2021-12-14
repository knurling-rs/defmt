#![no_std]
#![no_main]

use core::{marker::PhantomData, num};
use cortex_m_rt::entry;
use cortex_m_semihosting::debug;
use defmt::{Debug2Format, Display2Format, Format, Formatter};

use defmt_semihosting as _; // global logger

#[entry]
fn main() -> ! {
    defmt::info!("Hello!");
    defmt::info!("World!");
    defmt::info!("The answer is {=u8}", 42);
    defmt::info!("Hello {0=u8} {0=u8}!", 42);
    defmt::info!("Hello {1=u16} {0=u8} {2=bool}", 42u8, 256u16, false);
    defmt::info!("🍕 slice {=[u8]}", [3, 14]);
    defmt::info!("🍕 array {=[u8; 3]}", [3, 14, 1]);
    defmt::info!("float like a butterfly {=f32} {}", 5.67f32, 5.67f32);
    defmt::info!(
        "double like a butterfly {=f64} {}",
        5.000000000000067f64,
        5.000000000000067f64
    );
    defmt::info!("Hello {=u8}", 42u16 as u8);

    defmt::info!(
        "Hex lower {=i8:x}, {=i16:x}, {=i32:x}, {=i64:x}, {=i128:x}",
        -1,
        -2,
        -3,
        -4,
        -5
    );
    defmt::info!(
        "Hex lower {=i8:#x}, {=i16:#x}, {=i32:#x}, {=i64:#x}, {=i128:#x}",
        -1,
        -2,
        -3,
        -4,
        -5
    );
    defmt::info!(
        "Hex upper {=i8:X}, {=i16:X}, {=i32:X}, {=i64:X}, {=i128:X}",
        -1,
        -2,
        -3,
        -4,
        -5
    );
    defmt::info!(
        "Hex upper {=i8:#X}, {=i16:#X}, {=i32:#X}, {=i64:#X}, {=i128:#X}",
        -1,
        -2,
        -3,
        -4,
        -5
    );

    defmt::info!(
        "Hex unsigned {=u8:04x}, {=u16:#08X}, {=u32:04x}, {=u64:#010x}, {=u128:#X}",
        1,
        2,
        200_000,
        4,
        u128::max_value()
    );

    defmt::info!(
        "u64: 0 = {=u64}, 1 = {=u64}, MAX = {=u64}, MIN = {=u64}",
        0,
        1,
        u64::max_value(),
        u64::min_value()
    );

    defmt::info!(
        "i64: 0 = {=i64}, -1 = {=i64}, MAX = {=i64}, MIN = {=i64}",
        0,
        -1,
        i64::max_value(),
        i64::min_value()
    );

    defmt::info!(
        "isize: 0 = {=isize}, -1 = {=isize}, MAX = {=isize}, MIN = {=isize}",
        0,
        -1,
        isize::max_value(),
        isize::min_value()
    );
    defmt::info!(
        "isize: 0 = {=?}, -1 = {=?}, MAX = {=?}, MIN = {=?}",
        0,
        -1,
        isize::max_value(),
        isize::min_value()
    );
    defmt::info!("usize: 0 = {=usize}, MAX = {=usize}", 0, usize::max_value());
    defmt::info!("bitfields {0=0..3} {0=5..7}", 0b0110_0011_1101_0110u16);
    defmt::trace!("log trace");
    defmt::debug!("log debug");
    defmt::info!("log info");
    defmt::warn!("log warn");
    defmt::error!("log error");

    #[derive(Format)]
    struct S {
        x: u8,
        y: u16,
    }

    #[derive(Format)]
    struct X {
        y: Y,
    }

    #[derive(Format)]
    struct Y {
        z: u8,
    }

    defmt::info!("{=?}", S { x: 1, y: 256 });
    defmt::info!("{=?}", X { y: Y { z: 42 } });

    let interned = defmt::intern!("interned string");
    defmt::info!("&str = {=str}", "string slice");
    defmt::info!("&str = {=?}", "string slice");
    defmt::info!("&Str = {=istr}", interned);
    defmt::info!("&Str = {=?}", interned);

    #[derive(Format)]
    struct Arr {
        arr1: [u8; 1],
        arr0: [u8; 0],
        arr32: [u8; 32],
    }

    defmt::info!(
        "{=?}",
        Arr {
            arr1: [0x1f],
            arr0: [],
            arr32: [0x55; 32]
        }
    );

    let slice: &[u16] = &[256, 257, 258];
    defmt::info!("{=[?]}", slice);

    let ss: &[S] = &[S { x: 128, y: 256 }, S { x: 129, y: 257 }];
    defmt::info!("{=[?]}", ss);

    let xs: &[X] = &[X { y: Y { z: 128 } }, X { y: Y { z: 129 } }];
    defmt::info!("{=[?]}", xs);

    let slices: &[&[u16]] = &[&[256, 257, 258], &[259, 260]];
    defmt::info!("{=[?]}", slices);

    #[derive(Format)]
    enum E {
        A,
        B,
    }

    defmt::info!("e1={=?}", E::A);
    defmt::info!("e2={=?}", E::B);

    defmt::info!("e3={=?}", Some(42u8));
    defmt::info!("e4={=?}", None::<u8>);

    defmt::info!("e5={=?}", Ok::<u8, u16>(42u8));
    defmt::info!("e6={=?}", Err::<u8, u16>(256u16));

    defmt::info!("e7={=?}", Some(X { y: Y { z: 42 } }));

    #[derive(Format)]
    struct Flags {
        a: bool,
        b: bool,
        c: bool,
    }

    // issue 74
    defmt::info!(
        "{=bool} {=?}",
        true,
        Flags {
            a: true,
            b: false,
            c: true
        }
    );

    // issue #111
    defmt::info!("{=[?]}", [true, true, false]);

    // issue #209
    defmt::info!("usize slice: {=?}", &[1usize, 2, 3][..]);
    defmt::info!("isize slice: {=?}", &[-1isize, -2, -3][..]);

    /* issue #124 (start) */
    // plain generic struct
    {
        #[derive(Format)]
        struct S<T> {
            x: u8,
            y: T,
        }

        defmt::info!("{=?}", S { x: 42, y: 43u8 });
    }

    // generic struct with bounds
    {
        #[derive(Format)]
        struct S<T>
        where
            T: Copy,
        {
            x: u8,
            y: T,
        }

        defmt::info!("{=?}", S { x: 44, y: 45u8 });
    }

    // generic struct with `Option` field
    {
        #[derive(Format)]
        struct S<T>
        where
            T: Copy,
        {
            x: u8,
            y: Option<T>,
        }

        defmt::info!(
            "{=?}",
            S {
                x: 46,
                y: Some(47u8)
            }
        );
    }

    // generic struct with lifetimes and lifetime bounds
    {
        #[derive(Format)]
        struct S<'a, T>
        where
            T: 'a,
        {
            x: Option<&'a u8>,
            y: T,
        }

        defmt::info!(
            "{=?}",
            S {
                x: Some(&48),
                y: 49u8
            }
        );
    }

    // plain generic enum
    {
        #[derive(Format)]
        enum E<X, Y> {
            A,
            B(X),
            C { y: Y },
        }

        defmt::info!("{=?}", E::<u8, u8>::A);
        defmt::info!("{=?}", E::<u8, u8>::B(42));
        defmt::info!("{=?}", E::<u8, u8>::C { y: 43 });
    }

    // generic enum with bounds
    {
        #[derive(Format)]
        enum E<X, Y>
        where
            X: Copy,
        {
            A,
            B(X),
            C { y: Y },
        }

        defmt::info!("{=?}", E::<u8, u8>::A);
        defmt::info!("{=?}", E::<u8, u8>::B(44));
        defmt::info!("{=?}", E::<u8, u8>::C { y: 45 });
    }

    /* issue #124 (end) */
    // generic enum with `Option`/`Result` fields
    {
        #[derive(Format)]
        enum E<X, Y> {
            A,
            B(Option<X>),
            C { y: Result<Y, u8> },
        }

        defmt::info!("{=?}", E::<u8, u8>::A);
        defmt::info!("{=?}", E::<u8, u8>::B(Some(46)));
        defmt::info!("{=?}", E::<u8, u8>::C { y: Ok(47) });
    }

    // generic enum with lifetimes and lifetime bounds
    {
        #[derive(Format)]
        enum E<'a, T>
        where
            T: 'a,
        {
            A,
            B(Option<&'a u8>),
            C { y: T },
        }

        defmt::info!("{=?}", E::<u8>::A);
        defmt::info!("{=?}", E::<u8>::B(Some(&48)));
        defmt::info!("{=?}", E::C { y: 49u8 });
    }

    // slice + built-in enum
    defmt::info!("{=[?]}", &[None, Some(42u8)][..]);
    defmt::info!("{=[?]}", &[Ok(42u8), Err(43u8)][..]);

    // slice + user-defined enum
    {
        #[derive(Format)]
        enum E {
            A,
            B(u8),
        }
        defmt::info!("{=[?]}", &[E::A, E::B(42)][..]);
    }

    // slice + struct + built-in enum
    {
        #[derive(Format)]
        struct S {
            x: u8,
            y: Option<u8>,
        }

        defmt::info!(
            "{=[?]}",
            &[S { x: 42, y: None }, S { x: 43, y: Some(44) }][..]
        );
    }

    // slice + built-in enum + struct
    {
        #[derive(Format)]
        struct S {
            x: u8,
            y: u16,
        }

        defmt::info!("{=[?]}", &[None, Some(S { x: 42, y: 256 })][..]);
    }

    // slice + built-in enum + slice
    let s: &[u8] = &[42, 43];
    defmt::info!("{=[?]}", &[None, Some(s)][..]);

    defmt::info!("after nested log: {=?}", nested());

    // printing @ is now allowed
    defmt::info!("I can now print the @ symbol!");
    let interned = defmt::intern!("this is @n interned string");
    defmt::info!("@nd @lso vi@ interned strings: {=istr}", interned);

    // Tuples
    defmt::info!("empty tuple: {=?}", ());
    defmt::info!("tuple of ints: {=?}", (1, 2, 3));
    defmt::info!("nested tuple of ints: {=?}", (1, 2, (3, 4, 5), (6, 7, 8)));
    defmt::info!(
        "super nested tuples: {=?}",
        ((((((((),),),),),),), (((((((), (),),),),),),),)
    );
    defmt::info!("slice of tuples: {=?}", &[(1, 2), (3, 4), (5, 6)][..]);
    defmt::info!("tuple of slices: {=?}", (&[1, 2, 3][..], &[4, 5, 6][..]));
    defmt::info!("tuple of [u8;4]: {=?}", ([1u8, 2, 3, 4], [5u8, 6, 7, 8]));

    // Arrays of T: Format
    defmt::info!("[u8;0]: {=[?;0]}", [0u8; 0]);
    defmt::info!("[u8;4]: {=[?;4]}", [1u8, 2, 3, 4]);
    defmt::info!("[i8;4]: {=[?;4]}", [-1i8, 2, 3, -4]);
    defmt::info!(
        "[(u32,u32);4]: {=[?;4]}",
        [(1u32, 2u32), (3, 4), (5, 6), (7, 8)]
    );

    defmt::info!("[u8;0]: {=?}", [0u8; 0]);
    defmt::info!("[u8;4]: {=?}", [1u8, 2, 3, 4]);
    defmt::info!("[i8;4]: {=?}", [-1i8, 2, 3, -4]);
    defmt::info!("[u32;4]: {=?}", [1u32, 2, 3, 4]);
    defmt::info!("[i32;4]: {=?}", [-1i32, 2, 3, -4]);
    defmt::info!(
        "[[u32;4];4]: {=?}",
        [[1u32, 2, 3, 4], [2, 3, 4, 5], [3, 4, 5, 6], [4, 5, 6, 7]]
    );
    defmt::info!("[Option<u32>;4]: {=?}", [Some(1u32), None, Some(3), None]);
    defmt::info!(
        "[(u32,u32);4]: {=?}",
        [(1u32, 2u32), (3, 4), (5, 6), (7, 8)]
    );
    // No special-cased length, uses const-generic slice fallback
    defmt::info!("[u8; 33]: {}", [1; 33]);

    {
        #[derive(Format)]
        enum Single {
            A { fld: u8 },
        }

        defmt::info!("1-variant enum: {=?}", Single::A { fld: 123 });

        #[derive(Format)]
        enum Wrap {
            A(Single),
        }

        defmt::info!("wrapped: {=?}", Wrap::A(Single::A { fld: 200 }));
    }

    {
        // check that bools are compressed per *log frame*, not per `Format` impl

        #[derive(Format)]
        struct A(bool);

        #[derive(Format)]
        struct B(bool);

        defmt::info!(
            "{=?}, {=?}, {=?}",
            (A(true), B(true)),
            (A(false), B(true)),
            (A(true), B(false))
        );
    }

    {
        // issue #208

        #[derive(Format)]
        pub struct DhcpReprMin {
            pub broadcast: bool,
            pub a: [u8; 2],
        }

        let dhcp_repr = DhcpReprMin {
            broadcast: true,
            a: [1, 2],
        };

        defmt::info!("true, [1, 2]: {=?}", dhcp_repr);
    }

    {
        struct Inner(u8);

        impl Format for Inner {
            fn format(&self, f: Formatter) {
                defmt::write!(f, "inner value ({=u8})", self.0);
            }
        }

        // `write!` tests
        struct MyStruct(Inner);

        impl Format for MyStruct {
            fn format(&self, f: Formatter) {
                defmt::write!(f, "outer value ({=?})", self.0);
            }
        }

        defmt::info!(
            "nested `Format` impls using `write!`: {=?}",
            MyStruct(Inner(42)),
        );

        struct MyMultiStruct(u32);

        impl Format for MyMultiStruct {
            fn format(&self, f: Formatter) {
                defmt::write!(f, "MyMultiStruct@{=u32} ", self.0);
                if self.0 == 0 {
                    defmt::write!(f, "IS ZERO")
                } else {
                    defmt::write!(f, "IS NOT ZERO, division result: {=u32}", 100 / self.0)
                }
            }
        }

        defmt::info!(
            "manual `Format` impl with multiple `write!`: {=?}",
            MyMultiStruct(0)
        );
        defmt::info!(
            "manual `Format` impl with multiple `write!`: {=?}",
            MyMultiStruct(20)
        );
    }

    // Debug adapter
    {
        #[allow(dead_code)]
        #[derive(Clone, Copy, Debug)]
        struct S {
            x: i8,
            y: i16,
        }

        let s = S { x: -1, y: 2 };
        defmt::info!("{}", Debug2Format(&s));
        defmt::info!("{}", Debug2Format(&Some(s)));
        defmt::info!("{}", Debug2Format(&[s, s]));
        defmt::info!("{}", Debug2Format(&[Some(s), None]));
    }

    {
        struct SocketAddr {
            ip: [u8; 4],
            port: u16,
        }

        impl core::fmt::Display for SocketAddr {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(
                    f,
                    "{}.{}.{}.{}:{}",
                    self.ip[0], self.ip[1], self.ip[2], self.ip[3], self.port
                )
            }
        }

        let addr = SocketAddr {
            ip: [127, 0, 0, 1],
            port: 8888,
        };

        defmt::info!("{=?}", Display2Format(&addr));
    }

    defmt::info!(
        "i128: 0 = {=i128}, -1 = {=i128}, MAX = {=i128}, MIN = {=i128}",
        0,
        -1,
        i128::max_value(),
        i128::min_value()
    );

    defmt::info!(
        "u128: 0 = {=u128}, -1 = {=u128}, MAX = {=u128}, MIN = {=u128}",
        0,
        1,
        u128::max_value(),
        u128::min_value()
    );

    defmt::info!("{=?}", 340282366920938u128);
    defmt::info!("{=?}", -170141183460469i128);

    defmt::info!("Hello {=char}", '💜');
    defmt::info!("Hello {=char} & {=?}", '💜', '🍕');

    {
        #[rustfmt::skip]
        #[allow(dead_code)]
        #[derive(Format)]
        enum EnumLarge {
            A000, A001, A002, A003, A004, A005, A006, A007, A008, A009, A010, A011, A012, A013, A014, A015, A016, A017,
            A018, A019, A020, A021, A022, A023, A024, A025, A026, A027, A028, A029, A030, A031, A032, A033, A034, A035,
            A036, A037, A038, A039, A040, A041, A042, A043, A044, A045, A046, A047, A048, A049, A050, A051, A052, A053,
            A054, A055, A056, A057, A058, A059, A060, A061, A062, A063, A064, A065, A066, A067, A068, A069, A070, A071,
            A072, A073, A074, A075, A076, A077, A078, A079, A080, A081, A082, A083, A084, A085, A086, A087, A088, A089,
            A090, A091, A092, A093, A094, A095, A096, A097, A098, A099, A100, A101, A102, A103, A104, A105, A106, A107,
            A108, A109, A110, A111, A112, A113, A114, A115, A116, A117, A118, A119, A120, A121, A122, A123, A124, A125,
            A126, A127, A128, A129, A130, A131, A132, A133, A134, A135, A136, A137, A138, A139, A140, A141, A142, A143,
            A144, A145, A146, A147, A148, A149, A150, A151, A152, A153, A154, A155, A156, A157, A158, A159, A160, A161,
            A162, A163, A164, A165, A166, A167, A168, A169, A170, A171, A172, A173, A174, A175, A176, A177, A178, A179,
            A180, A181, A182, A183, A184, A185, A186, A187, A188, A189, A190, A191, A192, A193, A194, A195, A196, A197,
            A198, A199, A200, A201, A202, A203, A204, A205, A206, A207, A208, A209, A210, A211, A212, A213, A214, A215,
            A216, A217, A218, A219, A220, A221, A222, A223, A224, A225, A226, A227, A228, A229, A230, A231, A232, A233,
            A234, A235, A236, A237, A238, A239, A240, A241, A242, A243, A244, A245, A246, A247, A248, A249, A250, A251,
            A252, A253, A254, A255, A256, A257, A258, A259, A260, A261, A262, A263, A264, A265, A266, A267, A268, A269,
        }

        defmt::info!("EnumLarge::{=?}", EnumLarge::A051);
        defmt::info!("EnumLarge::{=?}", EnumLarge::A269);
    }

    {
        #[derive(Format)]
        struct S {
            x: &'static str,
        }

        defmt::info!("{}", S { x: "hi" });
    }

    {
        // #565 - should handle '|' symbol correctly
        struct State {}
        impl Format for State {
            fn format(&self, f: Formatter) {
                defmt::write!(f, "{=u8}|", 13_u8);
            }
        }

        defmt::info!("State: {}", State {});
    }

    {
        #[derive(Format)]
        struct S {
            x: PhantomData<u8>,
            y: u8,
        }

        defmt::info!(
            "{}",
            S {
                x: PhantomData,
                y: 42
            }
        );
    }

    defmt::info!(
        "bitfields \
        {0=120..128:x} \
        {0=112..120:b} \
        {0=80..88} \
        {0=48..64:a} \
        {=8..48:a}",
        0x9784_89AE_FF0C_5900_3432_6865_6C6C_6F00u128
    );

    let bytes: &[u8; 2] = b"Hi";
    let array_u16: &[u16; 2] = &[0xAF_FE, 0xC0_FE];

    defmt::info!("{=[u8]:a}", *bytes);
    defmt::info!("{=[?]:a}", *bytes);
    defmt::info!("{:a}", *bytes);
    defmt::info!("{=[?]:a}", *array_u16);

    {
        #[derive(Format)]
        struct Data<'a> {
            name: &'a [u8],
            value: bool,
        }

        let data = &[Data {
            name: b"Hi",
            value: true,
        }];
        defmt::info!("{=[?]:a}", *data);
    }

    // #341 - should output `true true`
    defmt::info!("{} {=bool}", True, true);

    // raw pointer
    defmt::info!("{}", 0xAABBCCDD as *const u8);
    defmt::info!("{}", 0xDDCCBBAA as *mut u8);

    // core::ops
    defmt::info!("{}", 1..2); // Range
    defmt::info!("{}", 1..); // RangeFrom
    defmt::info!("{}", ..2); // RangeTo
    defmt::info!("{}", ..); // RangeFull
    defmt::info!("{}", 1..=2); // RangeInclusive
    defmt::info!("{}", ..=2); // RangeToInclusive

    // core::iter
    defmt::info!("{}", [0, 1, 2].iter().zip([2, 1, 0].iter())); // Zip

    // core::slice
    defmt::info!("{}", [0, 1, 2].chunks_exact(1)); // ChunksExact
    defmt::info!("{}", [0, 1, 2].iter()); // ChunksExact
    defmt::info!("{}", [0, 1, 2].windows(1)); // Windows

    // core::num::NonZero*
    defmt::info!("{}", num::NonZeroI8::new(1).unwrap());
    defmt::info!("{}", num::NonZeroI16::new(1).unwrap());
    defmt::info!("{}", num::NonZeroI32::new(1).unwrap());
    defmt::info!("{}", num::NonZeroI64::new(1).unwrap());
    defmt::info!("{}", num::NonZeroI128::new(1).unwrap());
    defmt::info!("{}", num::NonZeroIsize::new(1).unwrap());
    defmt::info!("{}", num::NonZeroU8::new(1).unwrap());
    defmt::info!("{}", num::NonZeroU16::new(1).unwrap());
    defmt::info!("{}", num::NonZeroU32::new(1).unwrap());
    defmt::info!("{}", num::NonZeroU64::new(1).unwrap());
    defmt::info!("{}", num::NonZeroU128::new(1).unwrap());
    defmt::info!("{}", num::NonZeroUsize::new(1).unwrap());

    struct NotFormatType;
    defmt::info!("{}", 0xCCBBAADD as *mut NotFormatType);

    // tests for `defmt::flush()`
    defmt::info!("log data: {}", 0xABCD);
    defmt::info!("flush! 🚽");
    defmt::flush();
    defmt::info!("log more data! 🎉");

    defmt::info!("QEMU test finished!");

    loop {
        debug::exit(debug::EXIT_SUCCESS)
    }
}

#[derive(Format)]
struct NestedStruct {
    a: u8,
    b: u32,
}

fn nested() -> NestedStruct {
    defmt::info!("in nested {=u8}", 123);
    NestedStruct {
        a: 0xAA,
        b: 0x12345678,
    }
}

struct True;

impl Format for True {
    fn format(&self, fmt: Formatter<'_>) {
        defmt::write!(fmt, "{=bool}", true);
    }
}

// like `panic-semihosting` but doesn't print to stdout (that would corrupt the defmt stream)
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        debug::exit(debug::EXIT_FAILURE)
    }
}
