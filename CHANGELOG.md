# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

We have several packages which live in this repository. Changes are tracked separately.

* [defmt](#defmt)
* [defmt-macros](#defmt-macros)
* [defmt-print](#defmt-print)
* [defmt-decoder](#defmt-decoder)
* [defmt-parser](#defmt-parser)
* [defmt-rtt](#defmt-rtt)
* [defmt-itm](#defmt-itm)
* [defmt-semihosting](#defmt-semihosting)
* [panic-probe](#panic-probe)
* [defmt-test](#defmt-test)
* [defmt-test-macros](#defmt-test-macros)
* [defmt-json-schema](#defmt-json-schema)
* [defmt-elf2table](#defmt-elf2table)
* [defmt-logger](#defmt-logger)

## defmt

> A highly efficient logging framework that targets resource-constrained devices, like microcontrollers

[defmt-next]: https://github.com/knurling-rs/defmt/compare/defmt-v1.0.0...main
[defmt-v1.0.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-v1.0.0
[defmt-v0.3.100]: https://github.com/knurling-rs/defmt/releases/tag/defmt-v0.3.100
[defmt-v0.3.10]: https://github.com/knurling-rs/defmt/releases/tag/defmt-v0.3.10
[defmt-v0.3.9]: https://github.com/knurling-rs/defmt/releases/tag/defmt-v0.3.9
[defmt-v0.3.8]: https://github.com/knurling-rs/defmt/releases/tag/defmt-v0.3.8
[defmt-v0.3.7]: https://github.com/knurling-rs/defmt/releases/tag/defmt-v0.3.7
[defmt-v0.3.6]: https://github.com/knurling-rs/defmt/releases/tag/defmt-v0.3.6
[defmt-v0.3.5]: https://github.com/knurling-rs/defmt/releases/tag/defmt-v0.3.5
[defmt-v0.3.4]: https://github.com/knurling-rs/defmt/releases/tag/defmt-v0.3.4
[defmt-v0.3.3]: https://github.com/knurling-rs/defmt/releases/tag/defmt-v0.3.3
[defmt-v0.3.2]: https://github.com/knurling-rs/defmt/releases/tag/defmt-v0.3.2
[defmt-v0.3.1]: https://github.com/knurling-rs/defmt/releases/tag/defmt-v0.3.1
[defmt-v0.3.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-v0.3.0
[defmt-v0.2.3]: https://github.com/knurling-rs/defmt/releases/tag/defmt-v0.2.3
[defmt-v0.2.2]: https://github.com/knurling-rs/defmt/releases/tag/defmt-v0.2.2
[defmt-v0.2.1]: https://github.com/knurling-rs/defmt/releases/tag/defmt-v0.2.1
[defmt-v0.2.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-v0.2.0
[defmt-v0.1.3]: https://github.com/knurling-rs/defmt/releases/tag/defmt-v0.1.3
[defmt-v0.1.2]: https://github.com/knurling-rs/defmt/releases/tag/defmt-v0.1.2
[defmt-v0.1.1]: https://github.com/knurling-rs/defmt/releases/tag/defmt-v0.1.1
[defmt-v0.1.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-v0.1.0

### [defmt-next]

### [defmt-v1.0.0] (2025-01-01)

* [#909] First 1.0 stable release :tada:

### [defmt-v0.3.100] (2025-01-01)

* [#909] Re-exports defmt-1.0.0

### [defmt-v0.3.10] (2024-11-29)

* [#902] Minor change to Format impl for `core::panic::PanicInfo`, eliding a lifetime specifier to satisfy Clippy 1.83.
* [#899] Pin the defmt-macro crate to avoid incompatible versions being used together

### [defmt-v0.3.9] (2024-11-27)

* [#889] Add script for book hosting
* [#887] Fix interning example in the book
* [#884] Upgrade dependencies: notify is now at v7, thiserror is now at v2
* [#883] Mark decoder and parser not as unstable anymore
* [#880] Merge function calls emitted by the macro to save space.
* [#874] `defmt`: Fix doc test
* [#872] `defmt`: Add `expect!` as alias for `unwrap!` for discoverability
* [#871] Set MSRV to Rust 1.76
* [#869] `macros`: Add missing type hints
* [#865] `defmt`: Replace proc-macro-error with proc-macro-error2
* [#858] `defmt`: Implement "passthrough" trait impls for *2Format wrappers
* [#857] Add an octal display hint (`:o`)
* [#856] `defmt`: Add a `Format` impl for `PanicInfo` and related types.
* [#852] `CI`: Update mdbook to v0.4.40
* [#848] `decoder`: add optional one-line format
* [#847] `decoder`: Fix log format width specifier not working as expected
* [#845] `decoder`: fix println!() records being printed with formatting
* [#843] `defmt`: Sort IDs of log msgs by severity to allow runtime filtering by severity
* [#822] `CI`: Run `cargo semver-checks` on every PR

### [defmt-v0.3.8] (2024-05-17)

* [#840] `defmt`: Support pre-1.77
* [#839] `CI`: Fix tests
* [#838] `defmt`: Switch to Cargo instruction compatible with older versions of Cargo

### [defmt-v0.3.7] (2024-05-13 (yanked))

* [#831] `CI`: Fix CI
* [#830] `book`: Add section about feature-gated derive Format
* [#821] Clean up
* [#813] doc: add note for the alloc feature flag
* [#812] `defmt`: Add a feature to stop linking a default panic handler
* [#811] `book`: Add some examples for byte slice/array hints as well
* [#805] `defmt`: Drop ip_in_core feature and re-enable nightly snapshot tests

### [defmt-v0.3.6] (2024-02-05)

* [#804] `CI`: Remove mdbook strategy
* [#803] `CI`: Disable nightly qemu-snapshot tests
* [#789] `defmt`: Add support for new time-related display hints

### [defmt-v0.3.5] (2023-06-19)

* [#758] `defmt-print`: Tidy up
* [#757] `defmt-print`: Allow reading from a tcp port
* [#756] `CI`: Switch from bors to merge queue
* [#753] `demft` Add `Format` impls for `core::ptr::NonNull` and `fn(Args...) -> Ret` (up to 12 arguments)

### [defmt-v0.3.4] (2023-04-05)

* [#747] Bump wire version

### [defmt-v0.3.3] (2023-03-29 (yanked))

* [#744] `defmt-parser`: Clean and simplify
* [#743] `defmt-parser`: Simplify tests with `rstest`
* [#740] Snapshot tests for `core::net`
* [#739] `xtask`: Clean up
* [#737] `panic-probe`: Add `hard_fault()` for use in `defmt::panic_handler`
* [#733] `defmt`: Add formatting for `core::net` with the `ip_in_core` feature
* [#603] `defmt`: Raw pointers now print as `0x1234` instead of `1234`
* [#536] `defmt-parser`: Switch to using an enum for errors, and add some help text pointing you to the defmt docs if you use the wrong type specifier in a format string.

### [defmt-v0.3.2] (2022-05-31)

* [#669] Refine docs for `--json` flag

### [defmt-v0.3.1] (2022-03-10)

#### Added

* [#662] `#[derive(Format)]` now accepts attribute fields to format fields using the `Debug2Format` adapter instead of a `Format` implementation.
* [#661] Add tests for Cell types
* [#656] Implement `defmt::Format` for `Cell` and `RefCell`
* [#630] Add test instructions to README.md

#### Changed

* [#659] mark extern::acquire() and extern::release() as unsafe. this is not a breaking change; this is an internal API
* [#640] use crate [critical-section](https://crates.io/crates/critical-section) in defmt-rtt
* [#634] update ELF parsing dependencies
* [#633] make RTT buffer size configurable
* [#626] Make errror message more meaningful in case of version-mismatch

### [defmt-v0.3.0] (2021-11-09)

* [#618] Support #[ignore] attribute in defmt_test
* [#621] Readme Diagram: Replace duplicate defmt-itm with defmt-rtt
* [#617] Add display hint to output `u64` as ISO8601 time
* [#620] Tidy up: remove unused code and dependencies
* [#619] Update all crates to rust edition 2021! 🎉
* [#610] `defmt-print`: Log if malformed frame gets skipped
* [#547] Migration guide `v0.2.x` to `v0.3.0`
* [#604] defmt-test: `#[cfg(test)]` the `#[defmt_test::tests]` module
* [#616] Update user guide part of the book
* [#615] Document how to deal with backward compatibility breakage
* [#614] Bugfix: decoder breaks with pipe symbol
* [#605] Properly handle the `off` pseudo-level in presence of nested logging directives
* [#611] Fix `cargo doc`-warnings
* [#608] `decoder`: Fix that `defmt::println!` shows leading space when timestamps are disabled
* [#601] Move crate `defmt` to `defmt/`
* [#600] Run snapshot & backcompat tests in dev mode only
* [#519] Target-side `env_logger`-like env filter
* [#592] xtask: add backward compability test
* [#598] `defmt-print`: Recover from decoding-errors
* [#569] Add defmt `println!` macro
* [#580] Structure `defmt::export`
* [#594] panic-probe: use UDF instruction on nested panics
* [#591] Remove timestamps from snapshot test
* [#585] Add xtask option to run a single snapshot test by name
* [#589] Implement `Format` for arrays of any length
* [#587] Tweak inline attributes to remove machine code duplication
* [#574] Refactor rtt [1/2]
* [#584] Remove outdated doc "you may only call write! once"
* [#581] Add impl for `alloc::Layout`
* [#579] defmt-rtt: fix check for blocking RTT
* [#577] Fix typo in `cfg` of encoding-feature-`compile_error!`
* [#578] `qemu`: Allow dead code
* [#550] Added `defmt::flush()`
* [#570] Support referring to `Self` in bitflags constants
* [#568] Encoding docs.
* [#564] Make order of bitflags values deterministic
* [#561] Remove unused cortex-m-rt in panic-probe a=Dirbaio
* [#562] Remove call to fill in user survey from `README`
* [#560] Update cortex-m-rt crate from `0.6` to `0.7`
* [#557] Add impl for TryFromSliceError
* [#556] Add impl for TryFromIntError
* [#551] Display git version & date to introduction section
* [#540] Separate "crate version" from "wire format version
* [#545] Revert "`build.rs`: Obtain version from macro; simplify"
* [#539] Add optional rzCOBS encoding+framing
* [#518] `build.rs`: Obtain version from macro; simplify
* [#543] `CI`: Temporarily drop backward-compatibility check
* [#542] `snapshot-tests`: Test alternate hint for bitfields
* [#538] Fix wrong bit count in comment.
* [#537] `snapshot-tests`: Delete `:?` hint without impact
* [#531] refactor the `macros` crate
* [#534] Attribute test progress message to the test in question;
* [#535] Don't print leading space when timestamp is absent
* [#529] Refactor user-guide of `book`
* [#528] Support bitflags
* [#533] Adds add for user survey into readme.
* [#527] `book`: Add logo and support text to introduction
* [#526] `decoder`: Simplify tests
* [#359] Implement precedence of inner display hint
* [#523] Minimize dependencies
* [#508] [5/n] Format trait v2
* [#522] Replace `µs` hint with `us`
* [#521] [3/n] Remove u24
* [#516] `xtask`: Only install additional targets for tests that require them
* [#512] Add overwrite option for xtask cross results.
* [#514] extend raw pointer implementation to include !Format types
* [#513] book/duplicates.md: discriminator -> disambiguator
* [#507] [2/n] - Remove code-size-costly optimizations
* [#505] [1/n] - Logger trait v2.

### [defmt-v0.2.3] (2021-06-17)

#### Added

* [#499] Illustrate structure of the defmt crates
* [#503] Add alternate hint ('#')
* [#509] `impl Format for NonZero*`

#### Changed

* [#488] Structure `impl Format`s into multiple files
* [#496] Bump build-dep `semver` to `1.0`
* [#489] Structure lib
* [#500] book: fix leftover old formatting syntax; typos
* [#510] `CI`: Don't install MacOS dependency which is included by default

#### Fixed

* [#497] `macros`: match unused vars if logging is disabled

### [defmt-v0.2.2] (2021-05-20)

#### Added

* [#446] Add usage examples for `Debug2Format`, `Display2Format`
* [#464] `impl<T> Format for {*const, *mut} T where T: Format + ?Sized`
* [#472] `impl Format for` the core::{iter, ops, slice} structs
* [#473] `impl Format for` all the `Cow`s
* [#478] add `dbg!` macro

#### Changed

* [#477] Disable logging calls via conditional compilation when all defmt features are disabled

### [defmt-v0.2.1] (2021-03-08)

#### Added

* [#403] Add knurling logo to API docs

#### Fixed

* [#413] Fix docs-rs build, by disabling feature "unstable-test"
* [#427] Drop outdated note about `defmt v0.2.0` from book

### [defmt-v0.2.0] (2021-02-19)

#### Added

* [#284] Implement support for `i128` and `u128` types
* [#291] Allows using `defmt` on x86 platforms by making the test suite use an internal Cargo feature
* [#293] Make `defmt` attributes forward input attributes
* [#294] Permits `use` items inside `#[defmt_test::tests]` modules
* [#296] Allows skipping the `defmt` version check in order to make development easier
* [#302] `derive(Format)` now supports more than 256 variants
* [#304] impl `Format` for `char`s
* [#313] Add display hints
* [#323] Merge `Uxx(u64)` (`Ixx(i64)`) and `U128(u128)` (`u128(i128)`) data variants
* [#327] `impl<T> Format for PhantomData<T>`
* [#329] Document safety of implementation detail functions
* [#335] Add the `defmt-itm` crate
* [#338] Add `defmt-logger` and `defmt-print` crates
* [#343] Customizable timestamps
* [#347] Document the grammar of `defmt`s current format parameter syntax
* [#351] Allow tools to distinguish user-controlled format strings from generated ones
* [#354] Add `f64` support
* [#376] Make `defmt-logger` more configurable, remove `probe-run` strings
* [#377] `defmt-test`: support returning `Result` from tests
* [#382] `impl Format for Infallible`
* [#391] `impl Format for core::time::Duration`

#### Changed

* [#297] Improves the output formatting and includes a progress indicator
* [#299] Test embedded test runner (`defmt-test`) as part of our CI setup
* [#300] `#[derive]` now uses built-in primitive formatting for primitive references
* [#303] Employ the help of [bors]
* [#305] `Formatter` is now passed by value, i.e. consumed during formatting
* [#308] compile-fail test new `Formatter` move semantics
* [#312] `str` fields in structs are now treated as a native type by the encoder instead of going through the `Format` trait
* [#325] Update our UI tests to work with the latest stable release
* [#331] Add more compile-fail tests
* [#332] Improve `Format` trait docs
* [#333] Hide `Formatter`'s `inner` field
* [#334] Fix dead link in parser docs
* [#337] Improve diagnostics on double `write!`
* [#339] Make leb64 encoding fully safe (while at the same time reducing its code footprint)
* [#340] Stream `core::fmt` adapters
* [#345] Reduce code size by avoiding 64-bit arithmetic in LEB encoding
* [#350] `panic-probe` now uses `defmt`s `Display2Format` to log panic messages. In consequence, panic messages won't get truncated anymore.
* [#355] Clarify the docs on `Write::write`
* [#352] Do not display full version with `--help`. Thanks to [Javier-varez]!
* [#354] Support `f64` floating point numbers.
* [#355] Clarify docs on `Write::write`.
* [#363] Improve test coverage on basic `defmt` usage on `std` rust
* [#364] Split firmware code into separate workspace
* [#368] `defmt-itm`: Raise compile error on `armv6m`
* [#369] Move `bors.toml` to `.github/`
* [#371] Link to git version of `defmt` book
* [#372] Update `Printers` section in `defmt` book
* [#373] Improve information in `Cargo.toml`
* [#379] Make link to `defmt` book clickable
* [#380] Merge crates `elf2table` and `logger` into `decoder`
* [#383] `defmt-test`: Modify attributes in place and handle `#[cfg]`
* [#384] pin unstable path dependencies
* [#385] defmt_decoder: Skip allocation of datastructure for raw symbols of the table entries in `fn get_locations`
* [#386], [#392] Refactor decoder
  * rename `mod logger` to `log`
  * make `fn parse_*`, `fn get_locations`, `fn decode` methods of `struct Table`
  * various simplifications and restructuring of internal code
* [#387] CI: bump timeout to 20 minutes
* [#389] defmt_decoder: Bump deps `object` and `gimli`

#### Fixed

* [#301] Fix the nightly builds after a `linked_list_allocator` feature change
* [#310], [#311] remove the runtime check (and matching tests) if the `write!` macro was called multiple times as this can no longer happen since `write!` now consumes the `Formatter` due to [#305].
* [#321] ASCII hint (`:a`) is now respected when used together with the `Format` trait (`=?` and `=[?]`).
* [#342] Fix a data corruption issue when using `bool`s in `write!`
* [#357] Fix issue preventing `defmt` from compiling on MacOS.

### [defmt-v0.1.3] (2020-11-30)

#### Fixed

* [#290] fixed cross compilation to ARMv6-M and other targets that have no CAS (Compare-and-Swap)
  primitives when the "alloc" feature is enabled

### [defmt-v0.1.2] (2020-11-26)

#### Added

* [#263] [#276] add and document `write!` macro
* [#273] [#280] add and document `unwrap!` macro
* [#266] add `panic!`-like and `assert!`-like macros which will log the panic message using `defmt` and then call `core::panic!` (by default)
* [#267], [#281] add `Debug2Format` and `Display2Format` adapters
* [#279] started adding notes about feature availability (e.g. "defmt 0.1.2 and up")

#### Changed

* [#257] code size optimizations
* [#265] updated the 'how we deal with duplicated format strings' section of our [implementation notes]

[implementation notes]: https://defmt.ferrous-systems.com/design.html

#### Fixed

* [#264] `probe-run` doesn't panic if log message is not UTF-8
* [#269] fixes compiler error that was thrown when using `defmt::panic` within e.g. a match expression
* [#272] braces in format args passed to the new `defmt::panic!` and `defmt::assert!` macros do not cause unexpected errors anymore

### [defmt-v0.1.1] (2020-11-16)

#### Fixed

* [#259] crates.io version of `defmt` crates no longer require `git` to be built

### [defmt-v0.1.0] (2020-11-11)

Initial release

## defmt-macros

> Macros for [defmt](#defmt)

[defmt-macros-next]: https://github.com/knurling-rs/defmt/compare/defmt-macros-v1.0.0...main
[defmt-macros-v1.0.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-macros-v1.0.0
[defmt-macros-v0.4.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-macros-v0.4.0
[defmt-macros-v0.3.10]: https://github.com/knurling-rs/defmt/releases/tag/defmt-macros-v0.3.10
[defmt-macros-v0.3.9]: https://github.com/knurling-rs/defmt/releases/tag/defmt-macros-v0.3.9
[defmt-macros-v0.3.8]: https://github.com/knurling-rs/defmt/releases/tag/defmt-macros-v0.3.8
[defmt-macros-v0.3.7]: https://github.com/knurling-rs/defmt/releases/tag/defmt-macros-v0.3.7
[defmt-macros-v0.3.6]: https://github.com/knurling-rs/defmt/releases/tag/defmt-macros-v0.3.6
[defmt-macros-v0.3.5]: https://github.com/knurling-rs/defmt/releases/tag/defmt-macros-v0.3.5
[defmt-macros-v0.3.4]: https://github.com/knurling-rs/defmt/releases/tag/defmt-macros-v0.3.4
[defmt-macros-v0.3.3]: https://github.com/knurling-rs/defmt/releases/tag/defmt-macros-v0.3.3
[defmt-macros-v0.3.2]: https://github.com/knurling-rs/defmt/releases/tag/defmt-macros-v0.3.2
[defmt-macros-v0.3.1]: https://github.com/knurling-rs/defmt/releases/tag/defmt-macros-v0.3.1
[defmt-macros-v0.3.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-macros-v0.3.0
[defmt-macros-v0.2.3]: https://github.com/knurling-rs/defmt/releases/tag/defmt-macros-v0.2.3
[defmt-macros-v0.2.2]: https://github.com/knurling-rs/defmt/releases/tag/defmt-macros-v0.2.2
[defmt-macros-v0.2.1]: https://github.com/knurling-rs/defmt/releases/tag/defmt-macros-v0.2.1
[defmt-macros-v0.2.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-macros-v0.2.0
[defmt-macros-v0.1.1]: https://github.com/knurling-rs/defmt/releases/tag/defmt-macros-v0.1.1
[defmt-macros-v0.1.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-macros-v0.1.0

### [defmt-macros-next]

### [defmt-macros-v1.0.0] (2025-01-01)

* [#909] First 1.0 stable release :tada:

### [defmt-macros-v0.4.0] (2024-11-29)

* [#899] Just a major version bump to stop it being used by older defmt versions.

### [defmt-macros-v0.3.10] (2024-11-27)

### [defmt-macros-v0.3.9] (2024-05-14)

* [#835] `macros`: Fix some `defmt` crate name usage

### [defmt-macros-v0.3.8] (2024-05-13)

### [defmt-macros-v0.3.7] (2024-03-05)

### [defmt-macros-v0.3.6] (2023-08-01)

### [defmt-macros-v0.3.5] (2023-05-05)

* [#750] Add support for decoding wire format version 3

### [defmt-macros-v0.3.4] (2023-03-29)

### [defmt-macros-v0.3.3] (2022-10-07)

### [defmt-macros-v0.3.2] (2022-03-10)

### [defmt-macros-v0.3.1] (2021-11-26)

### [defmt-macros-v0.3.0] (2021-11-26)

### [defmt-macros-v0.2.3] (2021-06-21)

### [defmt-macros-v0.2.2] (2021-06-01)

### [defmt-macros-v0.2.1] (2021-05-21)

### [defmt-macros-v0.2.0] (2021-02-19)

### [defmt-macros-v0.1.1] (2020-11-26)

### [defmt-macros-v0.1.0] (2020-11-30)

Initial release

## defmt-print

> A tool that decodes defmt logs and prints them to the console

[defmt-print-next]: https://github.com/knurling-rs/defmt/compare/defmt-print-v1.0.0...main
[defmt-print-v1.0.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-print-v1.0.0
[defmt-print-v0.3.13]: https://github.com/knurling-rs/defmt/releases/tag/defmt-print-v0.3.13
[defmt-print-v0.3.12]: https://github.com/knurling-rs/defmt/releases/tag/defmt-print-v0.3.12
[defmt-print-v0.3.11]: https://github.com/knurling-rs/defmt/releases/tag/defmt-print-v0.3.11
[defmt-print-v0.3.10]: https://github.com/knurling-rs/defmt/releases/tag/defmt-print-v0.3.10
[defmt-print-v0.3.9]: https://github.com/knurling-rs/defmt/releases/tag/defmt-print-v0.3.9
[defmt-print-v0.3.8]: https://github.com/knurling-rs/defmt/releases/tag/defmt-print-v0.3.8
[defmt-print-v0.3.7]: https://github.com/knurling-rs/defmt/releases/tag/defmt-print-v0.3.7
[defmt-print-v0.3.6]: https://github.com/knurling-rs/defmt/releases/tag/defmt-print-v0.3.6
[defmt-print-v0.3.5]: https://github.com/knurling-rs/defmt/releases/tag/defmt-print-v0.3.5
[defmt-print-v0.3.4]: https://github.com/knurling-rs/defmt/releases/tag/defmt-print-v0.3.4
[defmt-print-v0.3.3]: https://github.com/knurling-rs/defmt/releases/tag/defmt-print-v0.3.3
[defmt-print-v0.3.2]: https://github.com/knurling-rs/defmt/releases/tag/defmt-print-v0.3.2
[defmt-print-v0.3.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-print-v0.3.1
[defmt-print-v0.2.2]: https://github.com/knurling-rs/defmt/releases/tag/defmt-print-v0.2.2
[defmt-print-v0.2.1]: https://github.com/knurling-rs/defmt/releases/tag/defmt-print-v0.2.1
[defmt-print-v0.2.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-print-v0.2.0

### [defmt-print-next]

### [defmt-print-v1.0.0] (2025-01-01)

* [#909] First 1.0 stable release :tada:

### [defmt-print-v0.3.13] (2024-11-27)

* [#807] Add `watch_elf` flag to allow ELF file reload without restarting `defmt-print`
* [#855] `defmt-print`: Now uses tokio to make tcp and stdin reads async (in preparation for a `watch elf` flag)

### [defmt-print-v0.3.12] (2024-05-13)

### [defmt-print-v0.3.11] (2024-03-05)

### [defmt-print-v0.3.10] (2023-10-04)

### [defmt-print-v0.3.9] (2023-08-01)

### [defmt-print-v0.3.8] (2023-08-01)

### [defmt-print-v0.3.7] (2023-05-05)

* [#750] Add support for decoding wire format version 3

### [defmt-print-v0.3.6] (2023-04-05)

### [defmt-print-v0.3.5] (2023-03-29)

### [defmt-print-v0.3.4] (2023-01-24)

* [#719] `defmt-print`: Fix panic

### [defmt-print-v0.3.3] (2022-10-07)

* [#703] `defmt-print`: Update to `clap 4.0`.
* [#682] `defmt-print`: exit when stdin is closed

### [defmt-print-v0.3.2] (2022-03-10)

### [defmt-print-v0.3.0] (2021-11-10)

### [defmt-print-v0.2.2] (2021-06-21)

### [defmt-print-v0.2.1] (2021-05-21)

### [defmt-print-v0.2.0] (2021-02-19)

### [defmt-print-v0.1.0]  (2021-01-15)

Initial release

## defmt-decoder

> Decodes defmt log frames

[defmt-decoder-next]: https://github.com/knurling-rs/defmt/compare/defmt-decoder-v1.0.0...main
[defmt-decoder-v1.0.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-decoder-v1.0.0
[defmt-decoder-v0.4.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-decoder-v0.4.0
[defmt-decoder-v0.3.11]: https://github.com/knurling-rs/defmt/releases/tag/defmt-decoder-v0.3.11
[defmt-decoder-v0.3.10]: https://github.com/knurling-rs/defmt/releases/tag/defmt-decoder-v0.3.10
[defmt-decoder-v0.3.9]: https://github.com/knurling-rs/defmt/releases/tag/defmt-decoder-v0.3.9
[defmt-decoder-v0.3.8]: https://github.com/knurling-rs/defmt/releases/tag/defmt-decoder-v0.3.8
[defmt-decoder-v0.3.7]: https://github.com/knurling-rs/defmt/releases/tag/defmt-decoder-v0.3.7
[defmt-decoder-v0.3.6]: https://github.com/knurling-rs/defmt/releases/tag/defmt-decoder-v0.3.6
[defmt-decoder-v0.3.5]: https://github.com/knurling-rs/defmt/releases/tag/defmt-decoder-v0.3.5
[defmt-decoder-v0.3.4]: https://github.com/knurling-rs/defmt/releases/tag/defmt-decoder-v0.3.4
[defmt-decoder-v0.3.3]: https://github.com/knurling-rs/defmt/releases/tag/defmt-decoder-v0.3.3
[defmt-decoder-v0.3.2]: https://github.com/knurling-rs/defmt/releases/tag/defmt-decoder-v0.3.2
[defmt-decoder-v0.3.1]: https://github.com/knurling-rs/defmt/releases/tag/defmt-decoder-v0.3.1
[defmt-decoder-v0.3.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-decoder-v0.3.0
[defmt-decoder-v0.2.2]: https://github.com/knurling-rs/defmt/releases/tag/defmt-decoder-v0.2.2
[defmt-decoder-v0.2.1]: https://github.com/knurling-rs/defmt/releases/tag/defmt-decoder-v0.2.1
[defmt-decoder-v0.2.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-decoder-v0.2.0
[defmt-decoder-v0.1.4]: https://github.com/knurling-rs/defmt/releases/tag/defmt-decoder-v0.1.4
[defmt-decoder-v0.1.3]: https://github.com/knurling-rs/defmt/releases/tag/defmt-decoder-v0.1.3
[defmt-decoder-v0.1.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-decoder-v0.1.0

### [defmt-decoder-next]

### [defmt-decoder-v1.0.0] (2025-01-01)

* [#909] First 1.0 stable release :tada:
* [#902] Minor change to `impl StreamDecoder` for `Raw` and `Rzcobs`, eliding a lifetime specifier to satisfy Clippy 1.83. No observable change.

### [defmt-decoder-v0.4.0] (2024-11-27)

### [defmt-decoder-v0.3.11] (2024-05-13)

### [defmt-decoder-v0.3.10] (2024-03-05)

### [defmt-decoder-v0.3.9] (2023-10-04)

### [defmt-decoder-v0.3.8] (2023-08-01)

### [defmt-decoder-v0.3.7] (2023-05-05)

### [defmt-decoder-v0.3.6] (2023-04-05)

### [defmt-decoder-v0.3.5] (2023-03-29)

### [defmt-decoder-v0.3.4] (2023-01-24)

* [#726] Remove difference in favor of dissimilar
* [#725] Replace chrono with time

### [defmt-decoder-v0.3.3] (2022-08-09)

* [#681] Make use of i/o locking being static since rust `1.61`.

### [defmt-decoder-v0.3.2] (2022-03-10)

### [defmt-decoder-v0.3.1] (2021-11-26)

### [defmt-decoder-v0.3.0] (2021-11-10)

### [defmt-decoder-v0.2.2] (2021-06-21)

### [defmt-decoder-v0.2.1] (2021-05-21)

### [defmt-decoder-v0.2.0] (2021-02-19)

### [defmt-decoder-v0.1.4] (2020-11-26)

### [defmt-decoder-v0.1.3] (2020-11-30)

### [defmt-decoder-v0.1.0] (2020-11-30)

Initial release

## defmt-parser

> Parsing library for defmt format strings

[defmt-parser-next]: https://github.com/knurling-rs/defmt/compare/defmt-parser-v1.0.0...main
[defmt-parser-v1.0.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-parser-v1.0.0
[defmt-parser-v0.4.1]: https://github.com/knurling-rs/defmt/releases/tag/defmt-parser-v0.4.1
[defmt-parser-v0.4.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-parser-v0.4.0
[defmt-parser-v0.3.4]: https://github.com/knurling-rs/defmt/releases/tag/defmt-parser-v0.3.4
[defmt-parser-v0.3.3]: https://github.com/knurling-rs/defmt/releases/tag/defmt-parser-v0.3.3
[defmt-parser-v0.3.2]: https://github.com/knurling-rs/defmt/releases/tag/defmt-parser-v0.3.2
[defmt-parser-v0.3.1]: https://github.com/knurling-rs/defmt/releases/tag/defmt-parser-v0.3.1
[defmt-parser-v0.3.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-parser-v0.3.0
[defmt-parser-v0.2.2]: https://github.com/knurling-rs/defmt/releases/tag/defmt-parser-v0.2.2
[defmt-parser-v0.2.1]: https://github.com/knurling-rs/defmt/releases/tag/defmt-parser-v0.2.1
[defmt-parser-v0.2.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-parser-v0.2.0
[defmt-parser-v0.1.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-parser-v0.1.0

### [defmt-parser-next]

### [defmt-parser-v1.0.0] (2025-01-01)

* [#909] First 1.0 stable release :tada:

### [defmt-parser-v0.4.1] (2024-11-27)

* [#897] Added its own README

### [defmt-parser-v0.4.0] (2024-11-27)

### [defmt-parser-v0.3.4] (2024-03-05)

### [defmt-parser-v0.3.3] (2023-05-05)

### [defmt-parser-v0.3.2] (2023-03-29)

### [defmt-parser-v0.3.1] (2022-03-10)

### [defmt-parser-v0.3.0] (2021-11-10)

### [defmt-parser-v0.2.2] (2021-06-21)

### [defmt-parser-v0.2.1] (2021-05-21)

### [defmt-parser-v0.2.0] (2021-02-19)

### [defmt-parser-v0.1.0] (2020-11-30)

Initial release

## defmt-rtt

> Transmit defmt log messages over the RTT (Real-Time Transfer) protocol

[defmt-rtt-next]: https://github.com/knurling-rs/defmt/compare/defmt-rtt-v1.0.0...main
[defmt-rtt-v1.0.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-rtt-v1.0.0
[defmt-rtt-v0.4.1]: https://github.com/knurling-rs/defmt/releases/tag/defmt-rtt-v0.4.1
[defmt-rtt-v0.4.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-rtt-v0.4.0
[defmt-rtt-v0.3.2]: https://github.com/knurling-rs/defmt/releases/tag/defmt-rtt-v0.3.2
[defmt-rtt-v0.3.1]: https://github.com/knurling-rs/defmt/releases/tag/defmt-rtt-v0.3.1
[defmt-rtt-v0.3.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-rtt-v0.3.0
[defmt-rtt-v0.2.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-rtt-v0.2.0
[defmt-rtt-v0.1.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-rtt-v0.1.0

### [defmt-rtt-next]

### [defmt-rtt-v1.0.0] (2025-01-01)

* [#909] First 1.0 stable release :tada:
* [#902] Use `core::ptr::addr_of_mut!` instead of `&mut` on mutable statics. No observable change.
* [#901] `defmt-rtt`: Update to critical-section 1.2

### [defmt-rtt-v0.4.1] (2024-05-13)

### [defmt-rtt-v0.4.0] (2022-10-07)

* [#701] `defmt-rtt`: Pre-relase cleanup
* [#695] `defmt-rtt`: Refactor rtt [3/2]
* [#689] `defmt-rtt`: Update to critical-section 1.0
* [#683] `defmt-rtt`: Make sure the whole RTT structure is in RAM

### [defmt-rtt-v0.3.2] (2022-03-10)

### [defmt-rtt-v0.3.1] (2021-11-26)

### [defmt-rtt-v0.3.0] (2021-11-26)

### [defmt-rtt-v0.2.0] (2021-02-20)

### [defmt-rtt-v0.1.0] (2020-11-30)

Initial release

## defmt-itm

> Transmit defmt log messages over the ITM (Instrumentation Trace Macrocell) stimulus port

[defmt-itm-next]: https://github.com/knurling-rs/defmt/compare/defmt-itm-v0.4.0...main
[defmt-itm-v0.4.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-itm-v0.4.0
[defmt-itm-v0.3.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-itm-v0.3.0
[defmt-itm-v0.2.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-itm-v0.2.0

### [defmt-itm-next]

### [defmt-itm-v0.4.0] (2025-01-01)

* [#909] Switch to using defmt-1.0
* [#902] Switch to using critical-section, and copy implementation over from defmt-rtt.

### [defmt-itm-v0.3.0] (2021-11-26)

### [defmt-itm-v0.2.0] (2021-02-20)

Initial release

## defmt-semihosting

> Transmit defmt log messages over the semihosting (Instrumentation Trace Macrocell) stimulus port

[defmt-semihosting-next]: https://github.com/knurling-rs/defmt/compare/defmt-semihosting-v0.2.0...main
[defmt-semihosting-v0.2.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-semihosting-v0.2.0
[defmt-semihosting-v0.1.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-semihosting-v0.1.0

### [defmt-semihosting-next]

### [defmt-semihosting-v0.2.0] (2025-01-01)

* [#909] Switch to using defmt-1.0

### [defmt-semihosting-v0.1.0] (2024-11-27)

Initial release

## panic-probe

> Panic handler that exits `probe-run` with an error code

[panic-probe-next]: https://github.com/knurling-rs/defmt/compare/panic-probe-v1.0.0...main
[panic-probe-v1.0.0]: https://github.com/knurling-rs/defmt/releases/tag/panic-probe-v1.0.0
[panic-probe-v0.3.2]: https://github.com/knurling-rs/defmt/releases/tag/panic-probe-v0.3.1
[panic-probe-v0.3.1]: https://github.com/knurling-rs/defmt/releases/tag/panic-probe-v0.3.0
[panic-probe-v0.3.0]: https://github.com/knurling-rs/defmt/releases/tag/panic-probe-v0.2.1
[panic-probe-v0.2.1]: https://github.com/knurling-rs/defmt/releases/tag/panic-probe-v0.2.0
[panic-probe-v0.2.0]: https://github.com/knurling-rs/defmt/releases/tag/panic-probe-v0.1.0
[panic-probe-v0.1.0]: https://github.com/knurling-rs/defmt/releases/tag/panic-probe-v0.0.0

### [panic-probe-next]

### [panic-probe-v1.0.0] (2025-01-01)

* [#909] Switch to using defmt-1.0

### [panic-probe-v0.3.2] (2024-05-13)

### [panic-probe-v0.3.1] (2023-03-29)

### [panic-probe-v0.3.0] (2021-11-26)

### [panic-probe-v0.2.1] (2021-09-17)

### [panic-probe-v0.2.0] (2021-02-20)

### [panic-probe-v0.1.0] (2020-11-30)

### panic-probe-v0.0.0

Initial release

## defmt-test

> A test harness for embedded devices

[defmt-test-next]:  https://github.com/knurling-rs/defmt/compare/defmt-test-v0.4.0...main
[defmt-test-v0.4.0]:  https://github.com/knurling-rs/defmt/releases/tag/defmt-test-v0.4.0
[defmt-test-v0.3.2]:  https://github.com/knurling-rs/defmt/releases/tag/defmt-test-v0.3.2
[defmt-test-v0.3.1]:  https://github.com/knurling-rs/defmt/releases/tag/defmt-test-v0.3.1
[defmt-test-v0.3.0]:  https://github.com/knurling-rs/defmt/releases/tag/defmt-test-v0.3.0
[defmt-test-v0.2.3]:  https://github.com/knurling-rs/defmt/releases/tag/defmt-test-v0.2.3
[defmt-test-v0.2.2]:  https://github.com/knurling-rs/defmt/releases/tag/defmt-test-v0.2.2
[defmt-test-v0.2.1]:  https://github.com/knurling-rs/defmt/releases/tag/defmt-test-v0.2.1
[defmt-test-v0.2.0]:  https://github.com/knurling-rs/defmt/releases/tag/defmt-test-v0.2.0
[defmt-test-v0.1.1]:  https://github.com/knurling-rs/defmt/releases/tag/defmt-test-v0.1.1
[defmt-test-v0.1.0]:  https://github.com/knurling-rs/defmt/releases/tag/defmt-test-v0.1.0

### [defmt-test-next]

### [defmt-test-v0.4.0] (2025-01-01)

* [#909] Switch to using defmt-1.0

### [defmt-test-v0.3.2] (2024-03-05)

### [defmt-test-v0.3.1] (2023-10-11)

### [defmt-test-v0.3.0] (2021-11-26)

### [defmt-test-v0.2.3] (2021-05-21)

### [defmt-test-v0.2.2] (2021-04-29)

### [defmt-test-v0.2.1] (2021-02-26)

### [defmt-test-v0.2.0] (2021-02-20)

### [defmt-test-v0.1.1] (2020-12-03)

### [defmt-test-v0.1.0] (2020-11-30)

Initial release

## defmt-test-macros

> Macros for defmt-test

[defmt-test-macros-next]: https://github.com/knurling-rs/defmt/compare/defmt-test-macros-v0.3.1...main
[defmt-test-macros-v0.3.1]: https://github.com/knurling-rs/defmt/releases/tag/defmt-test-macros-v0.3.1
[defmt-test-macros-v0.3.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-test-macros-v0.3.0
[defmt-test-macros-v0.2.1]: https://github.com/knurling-rs/defmt/releases/tag/defmt-test-macros-v0.2.1
[defmt-test-macros-v0.2.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-test-macros-v0.2.0
[defmt-test-macros-v0.1.1]: https://github.com/knurling-rs/defmt/releases/tag/defmt-test-macros-v0.1.1
[defmt-test-macros-v0.1.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-test-macros-v0.1.0

### [defmt-test-macros-next]

### [defmt-test-macros-v0.3.1] (2024-03-05)

### [defmt-test-macros-v0.3.0] (2021-11-26)

### [defmt-test-macros-v0.2.1] (2021-05-21)

### [defmt-test-macros-v0.2.0] (2021-02-26)

### [defmt-test-macros-v0.1.1] (2020-11-30)

### [defmt-test-macros-v0.1.0] (2020-11-30)

Initial release

## defmt-json-schema

> JSON schema for defmt

[defmt-json-schema-next]: https://github.com/knurling-rs/defmt/compare/defmt-json-schema-v0.1.0...main
[defmt-json-schema-v0.1.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-json-schema-v0.1.0

### [defmt-json-schema-next]

### [defmt-json-schema-v0.1.0] (2022-03-10)

Initial release

## defmt-elf2table

> Reads ELF metadata and builds a defmt interner table

Now defunct - lives in [defmt-decoder](#defmt-decoder)

[defmt-elf2table-v0.1.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-elf2table-v0.1.0

### [defmt-elf2table-v0.1.0] (2020-11-30)

Initial release

## defmt-logger

Now defunct - lives in [defmt-decoder](#defmt-decoder)

[defmt-logger-v0.1.0]: https://github.com/knurling-rs/defmt/releases/tag/defmt-logger-v0.1.0

### [defmt-logger-v0.1.0] (2021-01-15)

Initial release

---

[#909]: https://github.com/knurling-rs/defmt/pull/909
[#902]: https://github.com/knurling-rs/defmt/pull/902
[#901]: https://github.com/knurling-rs/defmt/pull/901
[#899]: https://github.com/knurling-rs/defmt/pull/899
[#897]: https://github.com/knurling-rs/defmt/pull/897
[#889]: https://github.com/knurling-rs/defmt/pull/889
[#887]: https://github.com/knurling-rs/defmt/pull/887
[#884]: https://github.com/knurling-rs/defmt/pull/884
[#883]: https://github.com/knurling-rs/defmt/pull/883
[#880]: https://github.com/knurling-rs/defmt/pull/880
[#874]: https://github.com/knurling-rs/defmt/pull/874
[#872]: https://github.com/knurling-rs/defmt/pull/872
[#871]: https://github.com/knurling-rs/defmt/pull/871
[#869]: https://github.com/knurling-rs/defmt/pull/869
[#865]: https://github.com/knurling-rs/defmt/pull/865
[#858]: https://github.com/knurling-rs/defmt/pull/858
[#857]: https://github.com/knurling-rs/defmt/pull/857
[#856]: https://github.com/knurling-rs/defmt/pull/856
[#855]: https://github.com/knurling-rs/defmt/pull/855
[#852]: https://github.com/knurling-rs/defmt/pull/852
[#848]: https://github.com/knurling-rs/defmt/pull/848
[#847]: https://github.com/knurling-rs/defmt/pull/847
[#845]: https://github.com/knurling-rs/defmt/pull/845
[#843]: https://github.com/knurling-rs/defmt/pull/843
[#840]: https://github.com/knurling-rs/defmt/pull/840
[#839]: https://github.com/knurling-rs/defmt/pull/839
[#838]: https://github.com/knurling-rs/defmt/pull/838
[#831]: https://github.com/knurling-rs/defmt/pull/831
[#822]: https://github.com/knurling-rs/defmt/pull/822
[#821]: https://github.com/knurling-rs/defmt/pull/821
[#813]: https://github.com/knurling-rs/defmt/pull/813
[#812]: https://github.com/knurling-rs/defmt/pull/812
[#811]: https://github.com/knurling-rs/defmt/pull/811
[#807]: https://github.com/knurling-rs/defmt/pull/807
[#805]: https://github.com/knurling-rs/defmt/pull/805
[#804]: https://github.com/knurling-rs/defmt/pull/804
[#803]: https://github.com/knurling-rs/defmt/pull/803
[#789]: https://github.com/knurling-rs/defmt/pull/789
[#758]: https://github.com/knurling-rs/defmt/pull/758
[#757]: https://github.com/knurling-rs/defmt/pull/757
[#756]: https://github.com/knurling-rs/defmt/pull/756
[#753]: https://github.com/knurling-rs/defmt/pull/753
[#750]: https://github.com/knurling-rs/defmt/pull/750
[#747]: https://github.com/knurling-rs/defmt/pull/747
[#744]: https://github.com/knurling-rs/defmt/pull/744
[#743]: https://github.com/knurling-rs/defmt/pull/743
[#740]: https://github.com/knurling-rs/defmt/pull/740
[#739]: https://github.com/knurling-rs/defmt/pull/739
[#737]: https://github.com/knurling-rs/defmt/pull/737
[#733]: https://github.com/knurling-rs/defmt/pull/733
[#726]: https://github.com/knurling-rs/defmt/pull/726
[#725]: https://github.com/knurling-rs/defmt/pull/725
[#719]: https://github.com/knurling-rs/defmt/pull/719
[#703]: https://github.com/knurling-rs/defmt/pull/703
[#701]: https://github.com/knurling-rs/defmt/pull/701
[#695]: https://github.com/knurling-rs/defmt/pull/695
[#689]: https://github.com/knurling-rs/defmt/pull/689
[#683]: https://github.com/knurling-rs/defmt/pull/683
[#682]: https://github.com/knurling-rs/defmt/pull/682
[#681]: https://github.com/knurling-rs/defmt/pull/681
[#662]: https://github.com/knurling-rs/defmt/pull/662
[#661]: https://github.com/knurling-rs/defmt/pull/661
[#659]: https://github.com/knurling-rs/defmt/pull/659
[#656]: https://github.com/knurling-rs/defmt/pull/656
[#640]: https://github.com/knurling-rs/defmt/pull/640
[#634]: https://github.com/knurling-rs/defmt/pull/634
[#633]: https://github.com/knurling-rs/defmt/pull/633
[#630]: https://github.com/knurling-rs/defmt/pull/630
[#626]: https://github.com/knurling-rs/defmt/pull/626
[#621]: https://github.com/knurling-rs/defmt/pull/621
[#620]: https://github.com/knurling-rs/defmt/pull/620
[#619]: https://github.com/knurling-rs/defmt/pull/619
[#618]: https://github.com/knurling-rs/defmt/pull/618
[#617]: https://github.com/knurling-rs/defmt/pull/617
[#616]: https://github.com/knurling-rs/defmt/pull/616
[#615]: https://github.com/knurling-rs/defmt/pull/615
[#614]: https://github.com/knurling-rs/defmt/pull/614
[#611]: https://github.com/knurling-rs/defmt/pull/611
[#610]: https://github.com/knurling-rs/defmt/pull/610
[#608]: https://github.com/knurling-rs/defmt/pull/608
[#605]: https://github.com/knurling-rs/defmt/pull/605
[#604]: https://github.com/knurling-rs/defmt/pull/604
[#603]: https://github.com/knurling-rs/defmt/pull/734
[#601]: https://github.com/knurling-rs/defmt/pull/601
[#600]: https://github.com/knurling-rs/defmt/pull/600
[#598]: https://github.com/knurling-rs/defmt/pull/598
[#594]: https://github.com/knurling-rs/defmt/pull/594
[#592]: https://github.com/knurling-rs/defmt/pull/592
[#591]: https://github.com/knurling-rs/defmt/pull/591
[#589]: https://github.com/knurling-rs/defmt/pull/589
[#587]: https://github.com/knurling-rs/defmt/pull/587
[#585]: https://github.com/knurling-rs/defmt/pull/585
[#584]: https://github.com/knurling-rs/defmt/pull/584
[#581]: https://github.com/knurling-rs/defmt/pull/581
[#580]: https://github.com/knurling-rs/defmt/pull/580
[#579]: https://github.com/knurling-rs/defmt/pull/579
[#578]: https://github.com/knurling-rs/defmt/pull/578
[#577]: https://github.com/knurling-rs/defmt/pull/577
[#574]: https://github.com/knurling-rs/defmt/pull/574
[#570]: https://github.com/knurling-rs/defmt/pull/570
[#569]: https://github.com/knurling-rs/defmt/pull/569
[#568]: https://github.com/knurling-rs/defmt/pull/568
[#564]: https://github.com/knurling-rs/defmt/pull/564
[#562]: https://github.com/knurling-rs/defmt/pull/562
[#561]: https://github.com/knurling-rs/defmt/pull/561
[#560]: https://github.com/knurling-rs/defmt/pull/560
[#557]: https://github.com/knurling-rs/defmt/pull/557
[#556]: https://github.com/knurling-rs/defmt/pull/556
[#551]: https://github.com/knurling-rs/defmt/pull/551
[#550]: https://github.com/knurling-rs/defmt/pull/550
[#547]: https://github.com/knurling-rs/defmt/pull/547
[#545]: https://github.com/knurling-rs/defmt/pull/545
[#543]: https://github.com/knurling-rs/defmt/pull/543
[#542]: https://github.com/knurling-rs/defmt/pull/542
[#540]: https://github.com/knurling-rs/defmt/pull/540
[#539]: https://github.com/knurling-rs/defmt/pull/539
[#538]: https://github.com/knurling-rs/defmt/pull/538
[#537]: https://github.com/knurling-rs/defmt/pull/537
[#536]: https://github.com/knurling-rs/defmt/pull/735
[#535]: https://github.com/knurling-rs/defmt/pull/535
[#534]: https://github.com/knurling-rs/defmt/pull/534
[#533]: https://github.com/knurling-rs/defmt/pull/533
[#531]: https://github.com/knurling-rs/defmt/pull/531
[#529]: https://github.com/knurling-rs/defmt/pull/529
[#528]: https://github.com/knurling-rs/defmt/pull/528
[#527]: https://github.com/knurling-rs/defmt/pull/527
[#526]: https://github.com/knurling-rs/defmt/pull/526
[#523]: https://github.com/knurling-rs/defmt/pull/523
[#522]: https://github.com/knurling-rs/defmt/pull/522
[#521]: https://github.com/knurling-rs/defmt/pull/521
[#519]: https://github.com/knurling-rs/defmt/pull/519
[#518]: https://github.com/knurling-rs/defmt/pull/518
[#516]: https://github.com/knurling-rs/defmt/pull/516
[#514]: https://github.com/knurling-rs/defmt/pull/514
[#513]: https://github.com/knurling-rs/defmt/pull/513
[#512]: https://github.com/knurling-rs/defmt/pull/512
[#510]: https://github.com/knurling-rs/defmt/pull/510
[#509]: https://github.com/knurling-rs/defmt/pull/509
[#508]: https://github.com/knurling-rs/defmt/pull/508
[#507]: https://github.com/knurling-rs/defmt/pull/507
[#505]: https://github.com/knurling-rs/defmt/pull/505
[#503]: https://github.com/knurling-rs/defmt/pull/503
[#500]: https://github.com/knurling-rs/defmt/pull/500
[#499]: https://github.com/knurling-rs/defmt/pull/499
[#497]: https://github.com/knurling-rs/defmt/pull/497
[#496]: https://github.com/knurling-rs/defmt/pull/496
[#489]: https://github.com/knurling-rs/defmt/pull/489
[#488]: https://github.com/knurling-rs/defmt/pull/488
[#478]: https://github.com/knurling-rs/defmt/pull/478
[#477]: https://github.com/knurling-rs/defmt/pull/477
[#473]: https://github.com/knurling-rs/defmt/pull/473
[#472]: https://github.com/knurling-rs/defmt/pull/472
[#464]: https://github.com/knurling-rs/defmt/pull/464
[#446]: https://github.com/knurling-rs/defmt/pull/446
[#427]: https://github.com/knurling-rs/defmt/pull/427
[#413]: https://github.com/knurling-rs/defmt/pull/413
[#403]: https://github.com/knurling-rs/defmt/pull/403
[#392]: https://github.com/knurling-rs/defmt/pull/392
[#391]: https://github.com/knurling-rs/defmt/pull/391
[#389]: https://github.com/knurling-rs/defmt/pull/389
[#387]: https://github.com/knurling-rs/defmt/pull/387
[#386]: https://github.com/knurling-rs/defmt/pull/386
[#385]: https://github.com/knurling-rs/defmt/pull/385
[#384]: https://github.com/knurling-rs/defmt/pull/384
[#383]: https://github.com/knurling-rs/defmt/pull/383
[#382]: https://github.com/knurling-rs/defmt/pull/382
[#380]: https://github.com/knurling-rs/defmt/pull/380
[#379]: https://github.com/knurling-rs/defmt/pull/379
[#377]: https://github.com/knurling-rs/defmt/pull/377
[#376]: https://github.com/knurling-rs/defmt/pull/376
[#373]: https://github.com/knurling-rs/defmt/pull/373
[#372]: https://github.com/knurling-rs/defmt/pull/372
[#371]: https://github.com/knurling-rs/defmt/pull/371
[#369]: https://github.com/knurling-rs/defmt/pull/369
[#368]: https://github.com/knurling-rs/defmt/pull/368
[#364]: https://github.com/knurling-rs/defmt/pull/364
[#363]: https://github.com/knurling-rs/defmt/pull/363
[#359]: https://github.com/knurling-rs/defmt/pull/359
[#357]: https://github.com/knurling-rs/defmt/pull/357
[#355]: https://github.com/knurling-rs/defmt/pull/355
[#354]: https://github.com/knurling-rs/defmt/pull/354
[#352]: https://github.com/knurling-rs/defmt/pull/352
[#351]: https://github.com/knurling-rs/defmt/pull/351
[#350]: https://github.com/knurling-rs/defmt/pull/350
[#347]: https://github.com/knurling-rs/defmt/pull/347
[#345]: https://github.com/knurling-rs/defmt/pull/345
[#343]: https://github.com/knurling-rs/defmt/pull/343
[#342]: https://github.com/knurling-rs/defmt/pull/342
[#340]: https://github.com/knurling-rs/defmt/pull/340
[#339]: https://github.com/knurling-rs/defmt/pull/339
[#338]: https://github.com/knurling-rs/defmt/pull/338
[#337]: https://github.com/knurling-rs/defmt/pull/337
[#335]: https://github.com/knurling-rs/defmt/pull/335
[#334]: https://github.com/knurling-rs/defmt/pull/334
[#333]: https://github.com/knurling-rs/defmt/pull/333
[#332]: https://github.com/knurling-rs/defmt/pull/332
[#331]: https://github.com/knurling-rs/defmt/pull/331
[#329]: https://github.com/knurling-rs/defmt/pull/329
[#327]: https://github.com/knurling-rs/defmt/pull/327
[#325]: https://github.com/knurling-rs/defmt/pull/325
[#323]: https://github.com/knurling-rs/defmt/pull/323
[#321]: https://github.com/knurling-rs/defmt/pull/321
[#313]: https://github.com/knurling-rs/defmt/pull/313
[#312]: https://github.com/knurling-rs/defmt/pull/312
[#311]: https://github.com/knurling-rs/defmt/pull/311
[#310]: https://github.com/knurling-rs/defmt/pull/310
[#308]: https://github.com/knurling-rs/defmt/pull/308
[#305]: https://github.com/knurling-rs/defmt/pull/305
[#304]: https://github.com/knurling-rs/defmt/pull/304
[#303]: https://github.com/knurling-rs/defmt/pull/303
[#302]: https://github.com/knurling-rs/defmt/pull/302
[#301]: https://github.com/knurling-rs/defmt/pull/301
[#300]: https://github.com/knurling-rs/defmt/pull/300
[#299]: https://github.com/knurling-rs/defmt/pull/299
[#297]: https://github.com/knurling-rs/defmt/pull/297
[#296]: https://github.com/knurling-rs/defmt/pull/296
[#294]: https://github.com/knurling-rs/defmt/pull/294
[#293]: https://github.com/knurling-rs/defmt/pull/293
[#291]: https://github.com/knurling-rs/defmt/pull/291
[#290]: https://github.com/knurling-rs/defmt/pull/290
[#284]: https://github.com/knurling-rs/defmt/pull/284
[#281]: https://github.com/knurling-rs/defmt/pull/281
[#280]: https://github.com/knurling-rs/defmt/pull/280
[#279]: https://github.com/knurling-rs/defmt/pull/279
[#276]: https://github.com/knurling-rs/defmt/pull/276
[#273]: https://github.com/knurling-rs/defmt/pull/273
[#272]: https://github.com/knurling-rs/defmt/pull/272
[#269]: https://github.com/knurling-rs/defmt/pull/269
[#267]: https://github.com/knurling-rs/defmt/pull/267
[#266]: https://github.com/knurling-rs/defmt/pull/266
[#265]: https://github.com/knurling-rs/defmt/pull/265
[#264]: https://github.com/knurling-rs/defmt/pull/264
[#263]: https://github.com/knurling-rs/defmt/pull/263
[#259]: https://github.com/knurling-rs/defmt/pull/259
[#257]: https://github.com/knurling-rs/defmt/pull/257
