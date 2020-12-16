// NOTE these tests should live in `defmt-macros` but the expansion of the macros defined there
// depend on `defmt` and `defmt` depends on `defmt-macros` -- the circular dependency may get in
// the way of `cargo test`

// NOTE string interning is mocked when testing so that it does not do real interning. Instead
// the "interner" always returns a **7-bit** `u8` value that's bumped on every interning operation.
//
// In practice, this means that the following operation:
// ```
// fn foo(f: &mut Formatter) {
//     write!(f, "Hello")
// }
// ```
// writes a *different* index on each call. With real interning this operation always writes the
// same index.
//
// `fetch_string_index` returns the current index of the mocked interner. Use this
// when writing the expected output of a unit test.
//
// ```
// let mut f = InternalFormatter::new();
// let index = defmt::export::fetch_string_index();
// foo(&mut f); // NOTE increases the interner index
// assert_eq!(f.bytes(), &[index]);
//
// let mut f = InternalFormatter::new();
// foo(&mut f);
// assert_eq!(f.bytes(), &[index.wrapping_add(1)]);
//                               ^^^^^^^^^^^^^^^ account for the previous `foo` call
// ```
//
// The mocked string index is thread local so you can run unit tests in parallel.
// `fetch_string_index` returns the thread-local interner index.
//
// Additional notes:
//
// - the mocked index is 7 bits so its LEB128 encoding is the input byte

use defmt::{export::fetch_string_index, write, Format, Formatter, InternalFormatter};

// Increase the 7-bit mocked interned index
fn inc(index: u8, n: u8) -> u8 {
    // NOTE(&) keep the highest bit at 0
    index.wrapping_add(n) & 0x7F
}

fn check_format_implementation(val: &(impl Format + ?Sized), expected_encoding: &[u8]) {
    let mut f = InternalFormatter::new();
    let g = Formatter {
        inner: &mut f,
    };
    val.format(g);
    f.finalize();
    assert_eq!(f.bytes(), expected_encoding);
}

#[test]
fn write() {
    let index = fetch_string_index();
    let ref mut f = InternalFormatter::new();
    let g = Formatter {
        inner: f,
    };

    write!(g, "The answer is {:u8}", 42);
    assert_eq!(
        f.bytes(),
        &[
            index, // "The answer is {:u8}",
            42,    // u8 value
        ]
    );

    let ref mut f2 = InternalFormatter::new();
    let g2 = Formatter {
        inner: f2,
    };
    write!(g2, "The answer is {:?}", 42u8);
    assert_eq!(
        f2.bytes(),
        &[
            inc(index, 1), // "The answer is {:?}"
            inc(index, 2), // "{:u8}" / impl Format for u8
            42,            // u8 value
        ]
    );
}

#[test]
fn booleans_max_num_bool_flags() {
    let index = fetch_string_index();
    let ref mut f = InternalFormatter::new();
    let g = Formatter {
        inner: f,
    };

    write!(
        g,
        "encode 8 bools {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool}",
        false, true, true, false, true, false, true, true
    );
    assert_eq!(
        f.bytes(),
        &[
            index,       // "encode 8 bools {:bool} {:bool} [...]",
            0b0110_1011, // compressed bools (dec value = 107)
        ]
    );
}

#[test]
fn booleans_less_than_max_num_bool_flags() {
    let index = fetch_string_index();
    let ref mut f = InternalFormatter::new();
    let g = Formatter {
        inner: f,
    };

    write!(
        g,
        "encode 3 bools {:bool} {:bool} {:bool}",
        false, true, true
    );
    assert_eq!(
        f.bytes(),
        &[
            index, // "encode 3 bools {:bool} {:bool} {:bool}",
            0b011, // compressed bools
        ]
    );
}

#[test]
fn booleans_more_than_max_num_bool_flags() {
    let index = fetch_string_index();
    let ref mut f = InternalFormatter::new();
    let g = Formatter {
        inner: f,
    };

    write!(g, "encode 9 bools {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool}",
           false, true, true, false, true, false, true, true, false, true);
    assert_eq!(
        f.bytes(),
        &[
            index,       // "encode 8 bools {:bool} {:bool} {:bool} [...]",
            0b0110_1011, // first 8 compressed bools
            0b01,        // final compressed bools
        ]
    );
}

#[test]
fn booleans_mixed() {
    let index = fetch_string_index();
    let ref mut f = InternalFormatter::new();
    let g = Formatter {
        inner: f,
    };

    write!(
        g,
        "encode mixed bools {:bool} {:bool} {:u8} {:bool}",
        true, false, 42, true
    );
    assert_eq!(
        f.bytes(),
        &[
            index, // "encode mixed bools {:bool} {:bool} {:u8} {:bool}",
            42u8,  // intermediate `42`
            0b101, // all compressed bools
        ]
    );
}

#[test]
fn booleans_mixed_no_trailing_bool() {
    let index = fetch_string_index();
    let ref mut f = InternalFormatter::new();
    let g = Formatter {
        inner: f,
    };

    write!(g, "encode mixed bools {:bool} {:u8}", false, 42);
    assert_eq!(
        f.bytes(),
        &[
            index, // "encode mixed bools {:bool} {:u8}",
            42u8, 0b0, // bool is put at the end of the args
        ]
    );
}

#[test]
fn bitfields_mixed() {
    let index = fetch_string_index();
    let ref mut f = InternalFormatter::new();
    let g = Formatter {
        inner: f,
    };

    write!(
        g,
        "bitfields {0:7..12}, {1:0..5}",
        0b1110_0101_1111_0000u16, 0b1111_0000u8
    );
    assert_eq!(
        f.bytes(),
        &[
            index, // bitfields {0:7..12}, {1:0..5}",
            0b1111_0000,
            0b1110_0101,   // u16
            0b1111_0000u8, // u8
        ]
    );
}

#[test]
fn bitfields_across_octets() {
    let index = fetch_string_index();
    let ref mut f = InternalFormatter::new();
    let g = Formatter {
        inner: f,
    };

    write!(g, "bitfields {0:0..7} {0:9..14}", 0b0110_0011_1101_0010u16);
    assert_eq!(
        f.bytes(),
        &[
            index, // bitfields {0:0..7} {0:9..14}",
            0b1101_0010,
            0b0110_0011, // u16
        ]
    );
}

#[test]
fn bitfields_truncate_lower() {
    let index = fetch_string_index();
    let ref mut f = InternalFormatter::new();
    let g = Formatter {
        inner: f,
    };

    write!(
        g,
        "bitfields {0:9..14}",
        0b0000_0000_0000_1111_0110_0011_1101_0010u32
    );
    assert_eq!(
        f.bytes(),
        &[
            index,       // bitfields {0:9..14}",
            0b0110_0011, // the first octet should have been truncated away
        ]
    );
}

#[test]
fn bitfields_assert_range_exclusive() {
    let index = fetch_string_index();
    let ref mut f = InternalFormatter::new();
    let g = Formatter {
        inner: f,
    };

    write!(g, "bitfields {0:6..8}", 0b1010_0101u8,);
    assert_eq!(
        f.bytes(),
        &[
            index, // "bitfields {0:6..8}"
            0b1010_0101
        ]
    );
}

#[test]
fn boolean_struct() {
    #[derive(Format)]
    struct X {
        y: bool,
        z: bool,
    }

    let index = fetch_string_index();
    check_format_implementation(
        &X { y: false, z: true },
        &[
            index, // "X {{ x: {:bool}, y: {:bool} }}"
            0b01,  // y and z compressed together
        ],
    )
}

#[test]
fn boolean_struct_mixed() {
    #[derive(Format)]
    struct X {
        y: bool,
        z: bool,
    }

    let index = fetch_string_index();
    let ref mut f = InternalFormatter::new();
    let g = Formatter {
        inner: f,
    };

    write!(
        g,
        "mixed formats {:bool} {:?}",
        true,
        X { y: false, z: true }
    );
    assert_eq!(
        f.bytes(),
        &[
            index,         // "mixed formats {:bool} {:?}",
            inc(index, 1), // "X {{ x: {:bool}, y: {:bool} }}"
            0b101,         // compressed struct bools
        ]
    );
}

#[test]
fn single_struct() {
    #[derive(Format)]
    struct X {
        y: u8,
        z: u16,
    }

    let index = fetch_string_index();
    check_format_implementation(
        &X { y: 1, z: 2 },
        &[
            index, // "X {{ x: {:u8}, y: {:u16} }}"
            1,     // x
            2,     // y.low
            0,     // y.high
        ],
    )
}

#[test]
fn single_struct_manual() {
    // Above `#[derive]`d impl should be equivalent to this:
    struct X {
        y: u8,
        z: u16,
    }

    impl Format for X {
        fn format(&self, f: Formatter) {
            defmt::write!(f, "X {{ x: {:u8}, y: {:u16} }}", self.y, self.z)
        }
    }

    let index = fetch_string_index();
    check_format_implementation(
        &X { y: 1, z: 2 },
        &[
            index, // "X {{ x: {:u8}, y: {:u16} }}"
            1,     // x
            2,     // y.low
            0,     // y.high
        ],
    )
}

#[test]
fn nested_struct() {
    #[derive(Format)]
    struct X {
        y: Y,
    }

    #[derive(Format)]
    struct Y {
        z: u8,
    }

    let val = 42;
    let index = fetch_string_index();
    check_format_implementation(
        &X { y: Y { z: val } },
        &[
            index,         // "X {{ y: {:?} }}"
            inc(index, 1), // "Y {{ z: {:u8} }}"
            val,
        ],
    );
}

#[test]
fn tuple_struct() {
    #[derive(Format)]
    struct Struct(u8, u16);

    let index = fetch_string_index();
    check_format_implementation(
        &Struct(0x1f, 0xaaaa),
        &[
            index, // "Struct({:u8}, {:u16})"
            0x1f,  // u8
            0xaa, 0xaa, // u16
        ],
    );
}

#[test]
fn c_like_enum() {
    #[derive(Format)]
    #[allow(dead_code)]
    enum Enum {
        A,
        B,
        C,
    }

    let index = fetch_string_index();
    check_format_implementation(
        &Enum::A,
        &[
            index, //
            0,     // `Enum::A`
        ],
    );
    let index = fetch_string_index();
    check_format_implementation(
        &Enum::B,
        &[
            index, //
            1,     // `Enum::B`
        ],
    );
}

#[test]
fn uninhabited_enum() {
    #[derive(Format)]
    enum Void {}
}

/// Tests that univariant enums do not encode the discriminant (since the variant in use is always
/// the same).
#[test]
fn univariant_enum() {
    #[derive(Format)]
    enum NoData {
        Variant,
    }

    let index = fetch_string_index();
    check_format_implementation(
        &NoData::Variant,
        &[
            index, //
        ],
    );

    #[derive(Format)]
    enum Data {
        Variant(u8, u16),
    }

    let index = fetch_string_index();
    check_format_implementation(
        &Data::Variant(0x1f, 0xaaaa),
        &[
            index, //
            0x1f,  // u8
            0xaa, 0xaa, // u16
        ],
    );
}

#[test]
fn nested_enum() {
    #[derive(Format)]
    #[allow(dead_code)]
    enum CLike {
        A,
        B,
        C,
    }

    #[derive(Format)]
    enum Inner {
        A(CLike, u8),
        _B,
    }

    #[derive(Format)]
    enum Outer {
        Variant1 { pre: u8, inner: Inner, post: u8 },
        Variant2,
        Variant3(Inner),
    }

    let index = fetch_string_index();
    check_format_implementation(
        &Outer::Variant1 {
            pre: 0xEE,
            inner: Inner::A(CLike::B, 0x07),
            post: 0xAB,
        },
        &[
            index,         //
            0,             // `Outer::Variant1`
            0xEE,          // u8 pre
            inc(index, 1), // `Inner`'s formatting string
            0,             // `Inner::A`
            inc(index, 2), // `CLike`'s formatting string
            1,             // `CLike::B`
            0x07,          // u8
            0xAB,          // u8 post
        ],
    );

    let index = fetch_string_index();
    check_format_implementation(
        &Outer::Variant2,
        &[
            index, //
            1,     // `Outer::Variant2`
        ],
    );

    let index = fetch_string_index();
    check_format_implementation(
        &Outer::Variant3(Inner::A(CLike::B, 0x07)),
        &[
            index,         //
            2,             // `Outer::Variant3`
            inc(index, 1), // `Inner`'s formatting string
            0,             // `Inner::A`
            inc(index, 2), // `CLike`'s formatting string
            1,             // `CLike::B`
            0x07,          // u8
        ],
    );
}

#[test]
fn slice() {
    let index = fetch_string_index();
    let val: &[u8] = &[23u8, 42u8];
    check_format_implementation(
        val,
        &[
            index,           // "{:[?]}"
            val.len() as u8, // length
            inc(index, 1),   // "{:u8}"
            23,              // val[0]
            42,              // val[1]
        ],
    )
}

#[test]
fn slice_of_usize() {
    let index = fetch_string_index();
    let val: &[usize] = &[23usize, 42];
    check_format_implementation(
        val,
        &[
            index,           // "{:[?]}"
            val.len() as u8, // length
            inc(index, 1),   // "{:usize}"
            23,              // val[0]
            42,              // val[1]
        ],
    )
}

#[test]
fn slice_of_bools() {
    let index = fetch_string_index();
    let val: &[bool] = &[true, true, false];
    check_format_implementation(
        val,
        &[
            index,           // "{:[?]}"
            val.len() as u8, // length
            inc(index, 1),   // "{:bool}"
            0b110,           // compressed bools: true, true, false
        ],
    )
}

#[test]
fn format_primitives() {
    let index = fetch_string_index();
    check_format_implementation(
        &42u8,
        &[
            index, // "{:u8}"
            42,
        ],
    );
    check_format_implementation(
        &42u16,
        &[
            inc(index, 1), // "{:u16}"
            42,
            0,
        ],
    );
    check_format_implementation(
        &513u16,
        &[
            inc(index, 2), // "{:u16}"
            1,
            2,
        ],
    );

    check_format_implementation(
        &42u32,
        &[
            inc(index, 3), // "{:u32}"
            42,
            0,
            0,
            0,
        ],
    );
    check_format_implementation(
        &513u32,
        &[
            inc(index, 4), // "{:u32}"
            1,
            2,
            0,
            0,
        ],
    );

    check_format_implementation(
        &5.13f32,
        &[
            inc(index, 5), // "{:f32}"
            246,
            40,
            164,
            64,
        ],
    );

    check_format_implementation(
        &42i8,
        &[
            inc(index, 6), // "{:i8}"
            42,
        ],
    );
    check_format_implementation(
        &-42i8,
        &[
            inc(index, 7), // "{:i8}"
            -42i8 as u8,
        ],
    );

    check_format_implementation(
        &None::<u8>,
        &[
            inc(index, 8), // "<option-format-string>"
            0,             // None discriminant
        ],
    );

    check_format_implementation(
        &Some(42u8),
        &[
            inc(index, 9),  // "<option-format-string>"
            1,              // Some discriminant
            inc(index, 10), // "{:u8}"
            42,             // Some.0 field
        ],
    );

    check_format_implementation(&-1isize, &[inc(index, 11), 0b0000_0001]);
    check_format_implementation(&-128isize, &[inc(index, 12), 0xff, 0b0000_0001]);

    check_format_implementation(
        &true,
        &[
            inc(index, 13), // "{:bool}"
            0b1,
        ],
    );

    check_format_implementation(
        &513u64,
        &[
            inc(index, 14), // "{:u64}"
            1,
            2,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
    );

    check_format_implementation(
        &-2i64,
        &[
            inc(index, 15), // "{:i64}"
            0xFE,
            0xFF,
            0xFF,
            0xFF,
            0xFF,
            0xFF,
            0xFF,
            0xFF,
        ],
    );

    check_format_implementation(
        &'a',
        &[
            inc(index, 16), // "{:char}"
            0x61,
            0x00,
            0x00,
            0x00,
        ],
    );
}

#[test]
fn istr() {
    let index = fetch_string_index();
    let interned = defmt::intern!("interned string contents");
    check_format_implementation(
        &interned,
        &[
            inc(index, 1), // "{:istr}"
            index,
        ],
    );
}

#[test]
fn format_arrays() {
    let index = fetch_string_index();
    let array: [u16; 0] = [];
    check_format_implementation(&array, &[index]);

    let index = fetch_string_index();
    let array: [u16; 3] = [1, 256, 257];
    check_format_implementation(
        &array,
        &[
            index,         // "{:[?;3]}"
            inc(index, 1), // "{:u16}"
            1,             // [0].low
            0,             // [0].high
            0,             // [1].low
            1,             // [1].high
            1,             // [2].low
            1,             // [2].high
        ],
    );
}

#[test]
fn format_slice_of_primitives() {
    let index = fetch_string_index();
    let slice: &[u16] = &[1, 256, 257];
    check_format_implementation(
        slice,
        &[
            index,             // "{:[?]}"
            slice.len() as u8, //
            inc(index, 1),     // "{:u16}"
            1,                 // [0].low
            0,                 // [0].high
            0,                 // [1].low
            1,                 // [1].high
            1,                 // [2].low
            1,                 // [2].high
        ],
    );
}

#[test]
fn format_slice_of_structs() {
    #[derive(Format)]
    struct X {
        y: Y,
    }

    #[derive(Format)]
    struct Y {
        z: u8,
    }

    let index = fetch_string_index();
    let slice: &[_] = &[X { y: Y { z: 42 } }, X { y: Y { z: 24 } }];
    check_format_implementation(
        slice,
        &[
            index,             // "{:[?]}"
            slice.len() as u8, //
            // first element
            inc(index, 1), // "X {{ y: {:?} }}"
            inc(index, 2), // "Y {{ z: {:u8} }}"
            42,            // [0].y.z
            // second element: no tags
            24, // [1].y.z
        ],
    );
}

#[test]
fn format_slice_of_slices() {
    let index = fetch_string_index();
    let slice: &[&[u16]] = &[&[256, 257], &[258, 259, 260]];
    check_format_implementation(
        slice,
        &[
            index,             // "{:[?]}"
            slice.len() as u8, //
            // first slice
            inc(index, 1), // "{:[?]}"
            slice[0].len() as u8,
            // its first element
            inc(index, 2), // "{:u16}"
            0,             // [0][0].low
            1,             // [0][0].high
            // its second element: no tag
            1, // [0][1].low
            1, // [0][1].high
            // second slice: no tags
            slice[1].len() as u8,
            2, // [1][0].low
            1, // [1][0].high
            3, // [1][1].low
            1, // [1][1].high
            4, // [1][2].low
            1, // [1][2].high
        ],
    );
}

#[test]
fn format_slice_enum_slice() {
    let index = fetch_string_index();
    let slice: &[Option<&[u8]>] = &[None, Some(&[42, 43])];
    check_format_implementation(
        slice,
        &[
            index,             // "{:[?]}"
            slice.len() as u8, //
            // first optional slice
            inc(index, 1), // "None|Some({:?})"
            0,             // discriminant
            // second optional slice
            // omitted: "None|Some({:?})" index
            1,             // discriminant
            inc(index, 2), // "{:[?]}" (the ? behind "Some({:?})")
            2,             // length of second optional slice
            inc(index, 3), // "{:u8}" (the ? behind "{:[?]}")
            42,
            // omitted: "{:u8}" index
            43,
        ],
    );
}

#[test]
fn format_slice_enum_generic_struct() {
    #[derive(Format)]
    struct S<T> {
        x: u8,
        y: T,
    }

    let index = fetch_string_index();
    let slice: &[Option<S<u8>>] = &[None, Some(S { x: 42, y: 43 })];
    check_format_implementation(
        slice,
        &[
            index,             // "{:[?]}"
            slice.len() as u8, //
            // first optional element
            inc(index, 1), // "None|Some({:?})"
            0,             // discriminant
            // second optional element
            // omitted: "None|Some({:?})" index
            1,             // discriminant
            inc(index, 2), // "S {{ x: {:u8}, y: {:?} }}" (the ? behind "Some({:?})")
            42,            // S.x
            inc(index, 3), // "{:u8}" (the ? behind S.y)
            43,            // S. y
        ],
    );
}

#[test]
fn derive_with_bounds() {
    #[derive(Format)]
    struct S<T: Copy> {
        val: T,
    }

    #[derive(Format)]
    struct S2<'a: 'b, 'b> {
        a: &'a u8,
        b: &'b u8,
    }

    let index = fetch_string_index();
    check_format_implementation(
        &S { val: 0 },
        &[
            index,         // "S {{ val: {:?} }}"
            inc(index, 1), // "{:i32}"
            0,
            0,
            0,
            0,
        ],
    );

    let index = fetch_string_index();
    check_format_implementation(
        &S2 { a: &1, b: &2 },
        &[
            index,         // "S2 { a: {:u8}, b: {:u8} }}"
            1,
            2,
        ],
    );
}

#[test]
fn format_bools() {
    #[derive(Format)]
    struct A(bool);

    #[derive(Format)]
    struct B(bool);

    let index = fetch_string_index();
    check_format_implementation(
        &(A(true), B(true)),
        &[
            index,         // "({:?}, {:?})"
            inc(index, 1), // "A({:bool})"
            inc(index, 2), // "B({:bool})"
            0b11,          // compressed bools
        ],
    );
}

#[test]
fn issue_208() {
    #[derive(Format)]
    struct DhcpReprMin {
        pub broadcast: bool,
        pub a: [u8; 2],
    }

    let dhcp_repr = DhcpReprMin {
        broadcast: true,
        a: [10, 10],
    };

    let index = fetch_string_index();
    check_format_implementation(
        &dhcp_repr,
        &[
            index,         // "DhcpReprMin {{ broadcast: {:bool}, a: {:?} }}"
            inc(index, 1), // "{:[?;2]}"
            inc(index, 2), // "{:u8}"
            10,            // a[0]
            10,            // a[1]
            1,             // compressed bools
        ],
    );
}

#[test]
fn enum_variants() {

    #[allow(dead_code)]
    #[derive(Format)]
    enum EnumSmall {
        A0,
        A1,
        A2
    }

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

    let e = EnumSmall::A2;

    let index = fetch_string_index();
    check_format_implementation(
        &e,
        &[
            index,
            2,
        ],
    );

    let e = EnumLarge::A269;

    let index = fetch_string_index();
    check_format_implementation(
        &e,
        &[
            index,
            13,
            1,
        ],
    );
}
