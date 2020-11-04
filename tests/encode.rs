// NOTE these tests should live in `defmt-macros` but the expansion of the macros defined there
// depend on `defmt` and `defmt` depends on `defmt-macros` -- the circular dependency may get in
// the way of `cargo test`

// NOTE string interning is mocked on x86 to aid testing so it does not do real interning. Instead
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
// let mut f = Formatter::new();
// let index = defmt::export::fetch_string_index();
// foo(&mut f); // NOTE increases the interner index
// assert_eq!(f.bytes(), &[index]);
//
// let mut f = Formatter::new();
// foo(&mut f);
// assert_eq!(f.bytes(), &[index.wrapping_add(1)]);
//                               ^^^^^^^^^^^^^^^ account for the previous `foo` call
// ```
//
// The mocked string index is thread local so you can run unit tests in parallel.
// `fetch_string_index` returns the thread-local interner index.
//
// The same mocking technique is applied to timestamps. There's a `fetch_timestamp`.
//
// Additional notes:
//
// - the mocked index is 7 bits so its LEB128 encoding is the input byte
// - the family of `info!` macros do nothing on x86; instead use `winfo!` which take a formatter
// argument like `write!`

use defmt::{
    export::{fetch_string_index, fetch_timestamp},
    winfo, Format, Formatter,
};

// Increase the 7-bit mocked interned index
fn inc(index: u8, n: u8) -> u8 {
    // NOTE(&) keep the highest bit at 0
    index.wrapping_add(n) & 0x7F
}

fn check_format_implementation(val: &(impl Format + ?Sized), expected_encoding: &[u8]) {
    let mut f = Formatter::new();
    val.format(&mut f);
    assert_eq!(f.bytes(), expected_encoding);
}

#[test]
fn info() {
    let index = fetch_string_index();
    let timestamp = fetch_timestamp();
    let mut f = Formatter::new();

    winfo!(f, "The answer is {:u8}", 42);
    assert_eq!(
        f.bytes(),
        &[
            index,     // "The answer is {:u8}",
            timestamp, //
            42,        // u8 value
        ]
    );

    let mut f = Formatter::new();
    winfo!(f, "The answer is {:?}", 42u8);
    assert_eq!(
        f.bytes(),
        &[
            inc(index, 1),     // "The answer is {:?}"
            inc(timestamp, 1), //
            inc(index, 2),     // "{:u8}" / impl Format for u8
            42,                // u8 value
        ]
    );
}

#[test]
fn booleans_max_num_bool_flags() {
    let index = fetch_string_index();
    let timestamp = fetch_timestamp();
    let mut f = Formatter::new();

    winfo!(
        f,
        "encode 8 bools {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool}",
        false,
        true,
        true,
        false,
        true,
        false,
        true,
        true
    );
    assert_eq!(
        f.bytes(),
        &[
            index,       // "encode 8 bools {:bool} {:bool} [...]",
            timestamp,   //
            0b0110_1011, // compressed bools (dec value = 107)
        ]
    );
}

#[test]
fn booleans_less_than_max_num_bool_flags() {
    let index = fetch_string_index();
    let timestamp = fetch_timestamp();
    let mut f = Formatter::new();

    winfo!(
        f,
        "encode 3 bools {:bool} {:bool} {:bool}",
        false,
        true,
        true
    );
    assert_eq!(
        f.bytes(),
        &[
            index,     // "encode 3 bools {:bool} {:bool} {:bool}",
            timestamp, //
            0b011,     // compressed bools
        ]
    );
}

#[test]
fn booleans_more_than_max_num_bool_flags() {
    let index = fetch_string_index();
    let timestamp = fetch_timestamp();
    let mut f = Formatter::new();

    winfo!(f, "encode 9 bools {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool}",
           false, true, true, false, true, false, true, true, false, true);
    assert_eq!(
        f.bytes(),
        &[
            index,       // "encode 8 bools {:bool} {:bool} {:bool} [...]",
            timestamp,   //
            0b0110_1011, // first 8 compressed bools
            0b01,        // final compressed bools
        ]
    );
}

#[test]
fn booleans_mixed() {
    let index = fetch_string_index();
    let timestamp = fetch_timestamp();
    let mut f = Formatter::new();

    winfo!(
        f,
        "encode mixed bools {:bool} {:bool} {:u8} {:bool}",
        true,
        false,
        42,
        true
    );
    assert_eq!(
        f.bytes(),
        &[
            index,     // "encode mixed bools {:bool} {:bool} {:u8} {:bool}",
            timestamp, //
            42u8, 0b101, // all compressed bools
        ]
    );
}

#[test]
fn booleans_mixed_no_trailing_bool() {
    let index = fetch_string_index();
    let timestamp = fetch_timestamp();
    let mut f = Formatter::new();

    winfo!(f, "encode mixed bools {:bool} {:u8}", false, 42);
    assert_eq!(
        f.bytes(),
        &[
            index,     // "encode mixed bools {:bool} {:u8}",
            timestamp, //
            42u8, 0b0, // bool is put at the end of the args
        ]
    );
}

#[test]
fn bitfields_mixed() {
    let index = fetch_string_index();
    let timestamp = fetch_timestamp();
    let mut f = Formatter::new();

    winfo!(
        f,
        "bitfields {0:7..12}, {1:0..5}",
        0b1110_0101_1111_0000u16,
        0b1111_0000u8
    );
    assert_eq!(
        f.bytes(),
        &[
            index, // bitfields {0:7..12}, {1:0..5}",
            timestamp,
            0b1111_0000,
            0b1110_0101,   // u16
            0b1111_0000u8, // u8
        ]
    );
}

#[test]
fn bitfields_across_octets() {
    let index = fetch_string_index();
    let timestamp = fetch_timestamp();
    let mut f = Formatter::new();

    winfo!(f, "bitfields {0:0..7} {0:9..14}", 0b0110_0011_1101_0010u16);
    assert_eq!(
        f.bytes(),
        &[
            index, // bitfields {0:0..7} {0:9..14}",
            timestamp,
            0b1101_0010,
            0b0110_0011, // u16
        ]
    );
}

#[test]
fn bitfields_truncate_lower() {
    let index = fetch_string_index();
    let timestamp = fetch_timestamp();
    let mut f = Formatter::new();

    winfo!(
        f,
        "bitfields {0:9..14}",
        0b0000_0000_0000_1111_0110_0011_1101_0010u32
    );
    assert_eq!(
        f.bytes(),
        &[
            index, // bitfields {0:9..14}",
            timestamp,
            0b0110_0011, // the first octet should have been truncated away
        ]
    );
}

#[test]
fn bitfields_assert_range_exclusive() {
    let index = fetch_string_index();
    let timestamp = fetch_timestamp();
    let mut f = Formatter::new();

    winfo!(f, "bitfields {0:6..8}", 0b1010_0101u8,);
    assert_eq!(
        f.bytes(),
        &[
            index, // "bitfields {0:6..8}"
            timestamp,
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
    let timestamp = fetch_timestamp();
    let mut f = Formatter::new();

    winfo!(
        f,
        "mixed formats {:bool} {:?}",
        true,
        X { y: false, z: true }
    );
    assert_eq!(
        f.bytes(),
        &[
            index, // "mixed formats {:bool} {:?}",
            timestamp,
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
            index,         // "S2 { a: {:?}, b: {:?} }}"
            inc(index, 1), // "{:u8}"
            1,
            inc(index, 2), // "{:u8}"
            2,
        ],
    );
}
