// NOTE these tests should live in `binfmt-macros` but the expansion of the macros defined there
// depend on `binfmt` and `binfmt` depends on `binfmt-macros` -- the circular dependency may get in
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
// let index = binfmt::export::fetch_string_index();
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

use binfmt::{
    export::{fetch_string_index, fetch_timestamp},
    winfo, Format, Formatter,
};

// Increase the 7-bit mocked interned index
fn inc(index: u8, n: u8) -> u8 {
    // NOTE(&) keep the highest bit at 0
    index.wrapping_add(n) & 0x7F
}

// CFI = Check Format Implementation
fn cfi(val: impl Format, expected_encoding: &[u8]) {
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
fn single_struct() {
    #[derive(Format)]
    struct X {
        y: u8,
        z: u16,
    }

    let index = fetch_string_index();
    cfi(
        X { y: 1, z: 2 },
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
    cfi(
        X { y: Y { z: val } },
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
    cfi(
        Struct(0x1f, 0xaaaa),
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
    cfi(
        Enum::A,
        &[
            index, //
            0,     // `Enum::A`
        ],
    );
    let index = fetch_string_index();
    cfi(
        Enum::B,
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

#[test]
fn univariant_enum() {
    #[derive(Format)]
    enum NoData {
        Variant,
    }

    let index = fetch_string_index();
    cfi(
        NoData::Variant,
        &[
            index, //
            0,     // `NoData::Variant`
        ],
    );

    #[derive(Format)]
    enum Data {
        Variant(u8, u16),
    }

    let index = fetch_string_index();
    cfi(
        Data::Variant(0x1f, 0xaaaa),
        &[
            index, //
            0,     // `Data::Variant`
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
    }

    #[derive(Format)]
    enum Outer {
        Variant1 { pre: u8, inner: Inner, post: u8 },
        Variant2,
        Variant3(Inner),
    }

    let index = fetch_string_index();
    cfi(
        Outer::Variant1 {
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
    cfi(
        Outer::Variant2,
        &[
            index, //
            1,     // `Outer::Variant2`
        ],
    );

    let index = fetch_string_index();
    cfi(
        Outer::Variant3(Inner::A(CLike::B, 0x07)),
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
    cfi(
        val,
        &[
            index, // todo
            2,     // val.len()
            23,     // val[0]
            42,     // val[1]
        ],
    )
}

#[test]
fn format_primitives() {
    let index = fetch_string_index();
    cfi(
        42u8,
        &[
            index, // "{:u8}"
            42,
        ],
    );
    cfi(
        42u16,
        &[
            inc(index, 1), // "{:u16}"
            42,
            0,
        ],
    );
    cfi(
        513u16,
        &[
            inc(index, 2), // "{:u16}"
            1,
            2,
        ],
    );

    cfi(
        42u32,
        &[
            inc(index, 3), // "{:u32}"
            42,
            0,
            0,
            0,
        ],
    );
    cfi(
        513u32,
        &[
            inc(index, 4), // "{:u32}"
            1,
            2,
            0,
            0,
        ],
    );

    cfi(
        42i8,
        &[
            inc(index, 5), // "{:i8}"
            42,
        ],
    );
    cfi(
        -42i8,
        &[
            inc(index, 6), // "{:i8}"
            -42i8 as u8,
        ],
    );

    cfi(
        None::<u8>,
        &[
            inc(index, 7), // "<option-format-string>"
            0,             // None discriminant
        ],
    );

    cfi(
        Some(42u8),
        &[
            inc(index, 8), // "<option-format-string>"
            1,             // Some discriminant
            inc(index, 9), // "{:u8}"
            42,            // Some.0 field
        ],
    );
}
