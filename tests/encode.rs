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

use binfmt::{export::fetch_string_index, Format, Formatter};

// Increase the 7-bit mocked interned index
fn inc(index: u8, n: u8) -> u8 {
    // NOTE(&) keep the highest bit at 0
    index.wrapping_add(n) & 0x7F
}

// CFI = Check Format Implementation
fn cfi(val: impl Format, bytes: &[u8]) {
    let mut f = Formatter::new();
    let index = fetch_string_index();
    val.format(&mut f);
    assert_eq!(f.bytes()[0], index); // e.g. "{:u8}"
    assert_eq!(&f.bytes()[1..], bytes);
}

#[test]
fn single_struct() {
    #[derive(Format)]
    struct X {
        y: u8,
        z: u16,
    }

    cfi(
        X { y: 1, z: 2 },
        &[
            1, // x
            2, // y.low
            0, // y.high
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
            // `index` = intern("X {{ y: {:?} }}") is checked in `cfi`
            inc(index, 1), // "Y {{ z: {:u8} }}"
            val,
        ],
    );
}

#[test]
fn format_primitives() {
    cfi(42u8, &[42]);
    cfi(42u16, &[42, 0]);
    cfi(513u16, &[1, 2]);

    cfi(42u32, &[42, 0, 0, 0]);
    cfi(513u32, &[1, 2, 0, 0]);

    cfi(42i8, &[42]);
    cfi(-42i8, &[-42i8 as u8]);

    cfi(
        None::<u8>,
        &[
            0, // None discriminant
        ],
    );
    let index = fetch_string_index();
    cfi(
        Some(42u8),
        &[
            // `index` = intern(<Option format string>) is checked in `cfi`
            1,             // Some discriminant
            inc(index, 1), // "{:u8}"
            42,            // Some.0 field
        ],
    );
}
