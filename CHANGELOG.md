# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## [v0.2.0] - 2021-02-19

### Added

- [#284] Implement support for `i128` and `u128` types
- [#291] Allows using `defmt` on x86 platforms by making the test suite use an internal Cargo feature
- [#293] Make `defmt` attributes forward input attributes
- [#294] Permits `use` items inside `#[defmt_test::tests]` modules
- [#296] Allows skipping the `defmt` version check in order to make development easier
- [#302] `derive(Format)` now supports more than 256 variants
- [#304] impl `Format` for `char`s
- [#313] Add display hints
- [#323] Merge `Uxx(u64)` (`Ixx(i64)`) and `U128(u128)` (`u128(i128)`) data variants
- [#327] `impl<T> Format for PhantomData<T>`
- [#329] Document safety of implementation detail functions
- [#335] Add the `defmt-itm` crate
- [#338] Add `defmt-logger` and `defmt-print` crates
- [#343] Customizable timestamps
- [#347] Document the grammar of `defmt`s current format parameter syntax
- [#351] Allow tools to distinguish user-controlled format strings from generated ones
- [#354] Add `f64` support
- [#376] Make `defmt-logger` more configurable, remove `probe-run` strings
- [#377] `defmt-test`: support returning `Result` from tests
- [#382] `impl Format for Infallible`
- [#391] `impl Format for core::time::Duration`

### Changed

- [#297] Improves the output formatting and includes a progress indicator
- [#299] Test embedded test runner (`defmt-test`) as part of our CI setup
- [#300] `#[derive]` now uses built-in primitive formatting for primitive references
- [#303] Employ the help of [bors]
- [#305] `Formatter` is now passed by value, i.e. consumed during formatting
- [#308] compile-fail test new `Formatter` move semantics
- [#312] `str` fields in structs are now treated as a native type by the encoder instead of going through the `Format` trait
- [#325] Update our UI tests to work with the latest stable release
- [#331] Add more compile-fail tests
- [#332] Improve `Format` trait docs
- [#333] Hide `Formatter`'s `inner` field
- [#334] Fix dead link in parser docs
- [#337] Improve diagnostics on double `write!`
- [#339] Make leb64 encoding fully safe (while at the same time reducing its code footprint)
- [#340] Stream `core::fmt` adapters
- [#345] Reduce code size by avoiding 64-bit arithmetic in LEB encoding
- [#350] `panic-probe` now uses `defmt`s `Display2Format` to log panic messages. In consequence, panic messages won't get truncated anymore.
- [#355] Clarify the docs on `Write::write`
- [#352] Do not display full version with `--help`. Thanks to [Javier-varez]!
- [#354] Support `f64` floating point numbers.
- [#355] Clarify docs on `Write::write`.
- [#361], [#367] Make clippy happy by improving code quality
- [#363] Improve test coverage on basic `defmt` usage on `std` rust
- [#364] Split firmware code into separate workspace
- [#368] `defmt-itm`: Raise compile error on `armv6m`
- [#369] Move `bors.toml` to `.github/`
- [#371] Link to git version of `defmt` book
- [#372] Update `Printers` section in `defmt` book
- [#373] Improve information in `Cargo.toml`
- [#379] Make link to `defmt` book clickable
- [#380] Merge crates `elf2table` and `logger` into `decoder`
- [#383] `defmt-test`: Modify attributes in place and handle `#[cfg]`
- [#384] pin unstable path dependencies
- [#385] defmt_decoder: Skip allocation of datastructure for raw symbols of the table entries in `fn get_locations`
- [#386], [#392] Refactor decoder
  - rename `mod logger` to `log`
  - make `fn parse_*`, `fn get_locations`, `fn decode` methods of `struct Table`
  - various simplifications and restructuring of internal code
- [#387] CI: bump timeout to 20 minutes 
- [#389] defmt_decoder: Bump deps `object` and `gimli`

### Fixed

- [#301] Fix the nightly builds after a `linked_list_allocator` feature change
- [#310], [#311] remove the runtime check (and matching tests) if the `write!` macro was called multiple times as this can no longer happen since `write!` now consumes the `Formatter` due to [#305].
- [#321] ASCII hint (`:a`) is now respected when used together with the `Format` trait (`=?` and `=[?]`).
- [#342] Fix a data corruption issue when using `bool`s in `write!`
- [#357] Fix issue preventing `defmt` from compiling on MacOS.

[#284]: https://github.com/knurling-rs/defmt/pull/284
[#291]: https://github.com/knurling-rs/defmt/pull/291
[#293]: https://github.com/knurling-rs/defmt/pull/293
[#294]: https://github.com/knurling-rs/defmt/pull/294
[#296]: https://github.com/knurling-rs/defmt/pull/296
[#297]: https://github.com/knurling-rs/defmt/pull/297
[#299]: https://github.com/knurling-rs/defmt/pull/299
[#300]: https://github.com/knurling-rs/defmt/pull/300
[#301]: https://github.com/knurling-rs/defmt/pull/301
[#302]: https://github.com/knurling-rs/defmt/pull/302
[#303]: https://github.com/knurling-rs/defmt/pull/303
[#304]: https://github.com/knurling-rs/defmt/pull/304
[#305]: https://github.com/knurling-rs/defmt/pull/305
[#308]: https://github.com/knurling-rs/defmt/pull/308
[#310]: https://github.com/knurling-rs/defmt/pull/310
[#311]: https://github.com/knurling-rs/defmt/pull/311
[#312]: https://github.com/knurling-rs/defmt/pull/312
[#313]: https://github.com/knurling-rs/defmt/pull/313
[#321]: https://github.com/knurling-rs/defmt/pull/321
[#323]: https://github.com/knurling-rs/defmt/pull/323
[#325]: https://github.com/knurling-rs/defmt/pull/325
[#327]: https://github.com/knurling-rs/defmt/pull/327
[#329]: https://github.com/knurling-rs/defmt/pull/329
[#331]: https://github.com/knurling-rs/defmt/pull/331
[#332]: https://github.com/knurling-rs/defmt/pull/332
[#333]: https://github.com/knurling-rs/defmt/pull/333
[#334]: https://github.com/knurling-rs/defmt/pull/334
[#335]: https://github.com/knurling-rs/defmt/pull/335
[#337]: https://github.com/knurling-rs/defmt/pull/337
[#338]: https://github.com/knurling-rs/defmt/pull/338
[#339]: https://github.com/knurling-rs/defmt/pull/339
[#340]: https://github.com/knurling-rs/defmt/pull/340
[#342]: https://github.com/knurling-rs/defmt/pull/342
[#343]: https://github.com/knurling-rs/defmt/pull/343
[#345]: https://github.com/knurling-rs/defmt/pull/345
[#347]: https://github.com/knurling-rs/defmt/pull/347
[#350]: https://github.com/knurling-rs/defmt/pull/350
[#351]: https://github.com/knurling-rs/defmt/pull/351
[#354]: https://github.com/knurling-rs/defmt/pull/354
[#355]: https://github.com/knurling-rs/defmt/pull/355
[#352]: https://github.com/knurling-rs/defmt/pull/352
[#354]: https://github.com/knurling-rs/defmt/pull/354
[#355]: https://github.com/knurling-rs/defmt/pull/355
[#357]: https://github.com/knurling-rs/defmt/pull/357
[#361]: https://github.com/knurling-rs/defmt/pull/361
[#363]: https://github.com/knurling-rs/defmt/pull/363
[#364]: https://github.com/knurling-rs/defmt/pull/364
[#368]: https://github.com/knurling-rs/defmt/pull/368
[#369]: https://github.com/knurling-rs/defmt/pull/369
[#371]: https://github.com/knurling-rs/defmt/pull/371
[#372]: https://github.com/knurling-rs/defmt/pull/372
[#373]: https://github.com/knurling-rs/defmt/pull/373
[#376]: https://github.com/knurling-rs/defmt/pull/376
[#377]: https://github.com/knurling-rs/defmt/pull/377
[#379]: https://github.com/knurling-rs/defmt/pull/379
[#380]: https://github.com/knurling-rs/defmt/pull/380
[#382]: https://github.com/knurling-rs/defmt/pull/382
[#383]: https://github.com/knurling-rs/defmt/pull/383
[#384]: https://github.com/knurling-rs/defmt/pull/384
[#385]: https://github.com/knurling-rs/defmt/pull/385
[#386]: https://github.com/knurling-rs/defmt/pull/386
[#387]: https://github.com/knurling-rs/defmt/pull/387
[#389]: https://github.com/knurling-rs/defmt/pull/389
[#391]: https://github.com/knurling-rs/defmt/pull/391
[#392]: https://github.com/knurling-rs/defmt/pull/392
[#396]: https://github.com/knurling-rs/defmt/pull/396

## [v0.1.3] - 2020-11-30

### Fixed

- [#290] fixed cross compilation to ARMv6-M and other targets that have no CAS (Compare-and-Swap)
  primitives when the "alloc" feature is enabled

[#290]: https://github.com/knurling-rs/defmt/pull/290

## [v0.1.2] - 2020-11-26

### Added

- [#263] [#276] add and document `write!` macro
- [#273] [#280] add and document `unwrap!` macro
- [#266] add `panic!`-like and `assert!`-like macros which will log the panic message using `defmt` and then call `core::panic!` (by default)
- [#267], [#281] add `Debug2Format` and `Display2Format` adapters
- [#279] started adding notes about feature availability (e.g. "defmt 0.1.2 and up")

[#263]: https://github.com/knurling-rs/defmt/pull/263
[#273]: https://github.com/knurling-rs/defmt/pull/273
[#276]: https://github.com/knurling-rs/defmt/pull/276
[#279]: https://github.com/knurling-rs/defmt/pull/279
[#266]: https://github.com/knurling-rs/defmt/pull/266
[#281]: https://github.com/knurling-rs/defmt/pull/281
[#267]: https://github.com/knurling-rs/defmt/pull/267
[#280]: https://github.com/knurling-rs/defmt/pull/280

### Changed

- [#257] code size optimizations
- [#265] updated the 'how we deal with duplicated format strings' section of our [implementation notes]

[#257]: https://github.com/knurling-rs/defmt/pull/257
[#265]: https://github.com/knurling-rs/defmt/pull/265
[implementation notes]: https://defmt.ferrous-systems.com/design.html

### Fixed

- [#264] `probe-run` doesn't panic if log message is not UTF-8
- [#269] fixes compiler error that was thrown when using `defmt::panic` within e.g. a match expression
- [#272] braces in format args passed to the new `defmt::panic!` and `defmt::assert!` macros do not cause unexpected errors anymore

[#264]: https://github.com/knurling-rs/defmt/pull/264
[#269]: https://github.com/knurling-rs/defmt/pull/269
[#272]: https://github.com/knurling-rs/defmt/pull/272

## [v0.1.1] - 2020-11-16

### Fixed

- [#259] crates.io version of `defmt` crates no longer require `git` to be built

[#259]: https://github.com/knurling-rs/defmt/pull/259

## v0.1.0 - 2020-11-11

Initial release

[Unreleased]: https://github.com/knurling-rs/defmt/compare/defmt-v0.2.0...main
[v0.2.0]: https://github.com/knurling-rs/defmt/compare/defmt-v0.1.3...defmt-v0.2.0
[v0.1.3]: https://github.com/knurling-rs/defmt/compare/defmt-v0.1.2...defmt-v0.1.3
[v0.1.2]: https://github.com/knurling-rs/defmt/compare/v0.1.1...defmt-v0.1.2
[v0.1.1]: https://github.com/knurling-rs/defmt/compare/v0.1.0...v0.1.1
