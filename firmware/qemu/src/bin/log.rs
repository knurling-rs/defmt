#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m_rt::entry;
use cortex_m_semihosting::debug;
use defmt::{consts, Debug2Format, Display2Format, Format, Formatter};

use defmt_semihosting as _; // global logger

#[entry]
fn main() -> ! {
    defmt::info!("Hello!");
    defmt::info!("World!");
    defmt::info!("The answer is {:u8}", 42);
    defmt::info!("Hello {0:u8} {0:u8}!", 42);
    defmt::info!("Hello {1:u16} {0:u8} {2:bool}", 42u8, 256u16, false);
    defmt::info!("🍕 slice {:[u8]}", [3, 14]);
    defmt::info!("🍕 array {:[u8; 3]}", [3, 14, 1]);
    defmt::info!("float like a butterfly {:f32}", 5.67f32);
    defmt::info!("Hello {:u8}", 42u16 as u8);

    defmt::info!(
        "u64: 0 = {:u64}, 1 = {:u64}, MAX = {:u64}, MIN = {:u64}",
        0,
        1,
        u64::max_value(),
        u64::min_value()
    );

    defmt::info!(
        "i64: 0 = {:i64}, -1 = {:i64}, MAX = {:i64}, MIN = {:i64}",
        0,
        -1,
        i64::max_value(),
        i64::min_value()
    );

    defmt::info!(
        "isize: 0 = {:isize}, -1 = {:isize}, MAX = {:isize}, MIN = {:isize}",
        0,
        -1,
        isize::max_value(),
        isize::min_value()
    );
    defmt::info!(
        "isize: 0 = {:?}, -1 = {:?}, MAX = {:?}, MIN = {:?}",
        0,
        -1,
        isize::max_value(),
        isize::min_value()
    );
    defmt::info!("usize: 0 = {:usize}, MAX = {:usize}", 0, usize::max_value());
    defmt::info!("bitfields {0:0..3} {0:5..7}", 0b0110_0011_1101_0110u16);

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

    defmt::info!("{:?}", S { x: 1, y: 256 });
    defmt::info!("{:?}", X { y: Y { z: 42 } });

    let interned = defmt::intern!("interned string");
    defmt::info!("&str = {:str}", "string slice");
    defmt::info!("&str = {:?}", "string slice");
    defmt::info!("&Str = {:istr}", interned);
    defmt::info!("&Str = {:?}", interned);

    #[derive(Format)]
    struct Arr {
        arr1: [u8; 1],
        arr0: [u8; 0],
        arr32: [u8; 32],
    }

    defmt::info!(
        "{:?}",
        Arr {
            arr1: [0x1f],
            arr0: [],
            arr32: [0x55; 32]
        }
    );

    let slice: &[u16] = &[256, 257, 258];
    defmt::info!("{:[?]}", slice);

    let ss: &[S] = &[S { x: 128, y: 256 }, S { x: 129, y: 257 }];
    defmt::info!("{:[?]}", ss);

    let xs: &[X] = &[X { y: Y { z: 128 } }, X { y: Y { z: 129 } }];
    defmt::info!("{:[?]}", xs);

    let slices: &[&[u16]] = &[&[256, 257, 258], &[259, 260]];
    defmt::info!("{:[?]}", slices);

    #[derive(Format)]
    enum E {
        A,
        B,
    }

    defmt::info!("e1={:?}", E::A);
    defmt::info!("e2={:?}", E::B);

    defmt::info!("e3={:?}", Some(42u8));
    defmt::info!("e4={:?}", None::<u8>);

    defmt::info!("e5={:?}", Ok::<u8, u16>(42u8));
    defmt::info!("e6={:?}", Err::<u8, u16>(256u16));

    defmt::info!("e7={:?}", Some(X { y: Y { z: 42 } }));

    #[derive(Format)]
    struct Flags {
        a: bool,
        b: bool,
        c: bool,
    }

    // issue 74
    defmt::info!(
        "{:bool} {:?}",
        true,
        Flags {
            a: true,
            b: false,
            c: true
        }
    );

    // issue #111
    defmt::info!("{:[?]}", [true, true, false]);

    // issue #209
    defmt::info!("usize slice: {:?}", &[1usize, 2, 3][..]);
    defmt::info!("isize slice: {:?}", &[-1isize, -2, -3][..]);

    /* issue #124 (start) */
    // plain generic struct
    {
        #[derive(Format)]
        struct S<T> {
            x: u8,
            y: T,
        }

        defmt::info!("{:?}", S { x: 42, y: 43u8 });
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

        defmt::info!("{:?}", S { x: 44, y: 45u8 });
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
            "{:?}",
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
            "{:?}",
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

        defmt::info!("{:?}", E::<u8, u8>::A);
        defmt::info!("{:?}", E::<u8, u8>::B(42));
        defmt::info!("{:?}", E::<u8, u8>::C { y: 43 });
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

        defmt::info!("{:?}", E::<u8, u8>::A);
        defmt::info!("{:?}", E::<u8, u8>::B(44));
        defmt::info!("{:?}", E::<u8, u8>::C { y: 45 });
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

        defmt::info!("{:?}", E::<u8, u8>::A);
        defmt::info!("{:?}", E::<u8, u8>::B(Some(46)));
        defmt::info!("{:?}", E::<u8, u8>::C { y: Ok(47) });
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

        defmt::info!("{:?}", E::<u8>::A);
        defmt::info!("{:?}", E::<u8>::B(Some(&48)));
        defmt::info!("{:?}", E::C { y: 49u8 });
    }

    // slice + built-in enum
    defmt::info!("{:[?]}", &[None, Some(42u8)][..]);
    defmt::info!("{:[?]}", &[Ok(42u8), Err(43u8)][..]);

    // slice + user-defined enum
    {
        #[derive(Format)]
        enum E {
            A,
            B(u8),
        }
        defmt::info!("{:[?]}", &[E::A, E::B(42)][..]);
    }

    // slice + struct + built-in enum
    {
        #[derive(Format)]
        struct S {
            x: u8,
            y: Option<u8>,
        }

        defmt::info!(
            "{:[?]}",
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

        defmt::info!("{:[?]}", &[None, Some(S { x: 42, y: 256 })][..]);
    }

    // slice + built-in enum + slice
    let s: &[u8] = &[42, 43];
    defmt::info!("{:[?]}", &[None, Some(s)][..]);

    defmt::info!("after nested log: {:?}", nested());

    // printing @ is now allowed
    defmt::info!("I can now print the @ symbol!");
    let interned = defmt::intern!("this is @n interned string");
    defmt::info!("@nd @lso vi@ interned strings: {:istr}", interned);

    // Tuples
    defmt::info!("empty tuple: {:?}", ());
    defmt::info!("tuple of ints: {:?}", (1, 2, 3));
    defmt::info!("nested tuple of ints: {:?}", (1, 2, (3, 4, 5), (6, 7, 8)));
    defmt::info!(
        "super nested tuples: {:?}",
        ((((((((),),),),),),), (((((((), (),),),),),),),)
    );
    defmt::info!("slice of tuples: {:?}", &[(1, 2), (3, 4), (5, 6)][..]);
    defmt::info!("tuple of slices: {:?}", (&[1, 2, 3][..], &[4, 5, 6][..]));
    defmt::info!("tuple of [u8;4]: {:?}", ([1u8, 2, 3, 4], [5u8, 6, 7, 8]));

    // Arrays of T: Format
    defmt::info!("[u8;0]: {:[?;0]}", [0u8; 0]);
    defmt::info!("[u8;4]: {:[?;4]}", [1u8, 2, 3, 4]);
    defmt::info!("[i8;4]: {:[?;4]}", [-1i8, 2, 3, -4]);
    defmt::info!(
        "[(u32,u32);4]: {:[?;4]}",
        [(1u32, 2u32), (3, 4), (5, 6), (7, 8)]
    );

    defmt::info!("[u8;0]: {:?}", [0u8; 0]);
    defmt::info!("[u8;4]: {:?}", [1u8, 2, 3, 4]);
    defmt::info!("[i8;4]: {:?}", [-1i8, 2, 3, -4]);
    defmt::info!("[u32;4]: {:?}", [1u32, 2, 3, 4]);
    defmt::info!("[i32;4]: {:?}", [-1i32, 2, 3, -4]);
    defmt::info!(
        "[[u32;4];4]: {:?}",
        [[1u32, 2, 3, 4], [2, 3, 4, 5], [3, 4, 5, 6], [4, 5, 6, 7]]
    );
    defmt::info!("[Option<u32>;4]: {:?}", [Some(1u32), None, Some(3), None]);
    defmt::info!(
        "[(u32,u32);4]: {:?}",
        [(1u32, 2u32), (3, 4), (5, 6), (7, 8)]
    );

    {
        #[derive(Format)]
        enum Single {
            A { fld: u8 },
        }

        defmt::info!("1-variant enum: {:?}", Single::A { fld: 123 });

        #[derive(Format)]
        enum Wrap {
            A(Single),
        }

        defmt::info!("wrapped: {:?}", Wrap::A(Single::A { fld: 200 }));
    }

    {
        // check that bools are compressed per *log frame*, not per `Format` impl

        #[derive(Format)]
        struct A(bool);

        #[derive(Format)]
        struct B(bool);

        defmt::info!(
            "{:?}, {:?}, {:?}",
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

        defmt::info!("true, [1, 2]: {:?}", dhcp_repr);
    }

    {
        struct Inner(u8);

        impl Format for Inner {
            fn format(&self, f: &mut Formatter) {
                defmt::write!(f, "inner value ({:u8})", self.0);
            }
        }

        // `write!` tests
        struct MyStruct(Inner);

        impl Format for MyStruct {
            fn format(&self, f: &mut Formatter) {
                defmt::write!(f, "outer value ({:?})", self.0);
            }
        }

        defmt::info!(
            "nested `Format` impls using `write!`: {:?}",
            MyStruct(Inner(42)),
        );
    }

    // Debug adapter
    {
        #[derive(Clone, Copy, Debug)]
        struct S {
            x: i8,
            y: i16,
        }

        let s = S { x: -1, y: 2 };
        defmt::info!("{:?}", Debug2Format::<consts::U128>(&s));
        defmt::info!("{:?}", Debug2Format::<consts::U128>(&Some(s)));
        defmt::info!("{:?}", Debug2Format::<consts::U128>(&[s, s]));
        defmt::info!("{:?}", Debug2Format::<consts::U128>(&[Some(s), None]));
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

        defmt::info!("{:?}", Display2Format::<consts::U32>(&addr));
    }

    defmt::info!(
        "i128: 0 = {:i128}, -1 = {:i128}, MAX = {:i128}, MIN = {:i128}",
        0,
        -1,
        i128::max_value(),
        i128::min_value()
    );

    defmt::info!(
        "u128: 0 = {:u128}, -1 = {:u128}, MAX = {:u128}, MIN = {:u128}",
        0,
        1,
        u128::max_value(),
        u128::min_value()
    );

    defmt::info!("{:?}", 340282366920938u128);
    defmt::info!("{:?}", -170141183460469i128);

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
    defmt::info!("in nested {:u8}", 123);
    NestedStruct {
        a: 0xAA,
        b: 0x12345678,
    }
}

#[defmt::timestamp]
fn timestamp() -> u64 {
    // monotonic counter
    static COUNT: AtomicU32 = AtomicU32::new(0);
    COUNT.fetch_add(1, Ordering::Relaxed) as u64
}

// like `panic-semihosting` but doesn't print to stdout (that would corrupt the defmt stream)
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        debug::exit(debug::EXIT_FAILURE)
    }
}
