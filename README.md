# `defmt-test`

> a test harness for embedded devices

## Basic usage

0. If you don't have a project setup yet, start from the [`app-template`] 


[`app-template`]: https://github.com/knurling-rs/app-template

1. In the **testsuite** crate, add `defmt-test` to the dependencies:

``` toml
# testsuite/Cargo.toml
[dependencies.defmt-test]
git = "https://github.com/knurling-rs/defmt-test"
branch = "main"
```

2. Within the `testsuite/tests/test.rs` file, create a `tests` module and mark it with the `#[defmt_test::tests]` attribute. Within that module write `std`-like unit tests: functions marked with the `#[test]` attribute.

``` rust
// testsuite/tests/test.rs

#[defmt_test::tests]
mod tests {
   #[test]
   fn assert_true() {
       assert!(true)
   }

   #[test]
   fn assert_false() {
       assert!(false)
   }
}
```

3. Run `cargo test -p testsuite` to run the unit tests

``` console
$ cargo test -p testsuite
0.000000 INFO            | running assert_true ..
0.000001 INFO            | .. assert_true ok
0.000002 INFO            | running assert_false ..
0.000003 ERROR           | panicked at 'assertion failed: false', testsuite/tests/test.rs:15:9
└─ ~/.cargo/git/checkouts/probe-run-31a04fec2ca67672/d81788c/panic-probe/src/lib.rs:139
stack backtrace:
   0: 0x000016cc - HardFaultTrampoline
      <exception entry>
   1: 0x00000706 - __udf
   2: 0x00001450 - cortex_m::asm::udf
   3: 0x0000147a - rust_begin_unwind
   4: 0x00000878 - core::panicking::panic_fmt
   5: 0x0000081a - core::panicking::panic
   6: 0x00000342 - test::tests::assert_false
   7: 0x0000029a - main
   8: 0x000000fa - Reset
```

NOTE unit tests will be executed sequentially

## Adding state

An `#[init]` function can be written within the `#[tests]` module.
This function will be executed before all unit tests and its return value, the test suite *state*, can be passed to unit tests as an argument.

``` rust
// state shared across unit tests
struct MyState {
    flag: bool,
}

#[defmt_test::tests]
mod tests {
    #[init]
    fn init() -> super::MyState {
        // state initial value
        super::MyState {
            flag: true,
        }
    }

    // this unit test doesn't access the state
    #[test]
    fn assert_true() {
        assert!(true);
    }

    // but this test does
    #[test]
    fn assert_flag(state: &mut super::MyState) {
        assert!(state.flag)
    }
}
```

``` console
$ cargo test -p testsuite
0.000000 INFO            | running assert_true ..
0.000001 INFO            | .. assert_true ok
0.000002 INFO            | running assert_flag ..
0.000003 INFO            | .. assert_flag ok
```

## Support

`defmt-test` is part of the [Knurling] project, [Ferrous Systems]' effort at
improving tooling used to develop for embedded systems.

If you think that our work is useful, consider sponsoring it via [GitHub
Sponsors].

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as above, without any additional terms or conditions.

[Knurling]: https://github.com/knurling-rs/meta
[Ferrous Systems]: https://ferrous-systems.com/
[GitHub Sponsors]: https://github.com/sponsors/knurling-rs
