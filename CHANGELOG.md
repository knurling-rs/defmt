# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

- [#703]: `defmt-print`: Update to `clap 4.0`.
- [#701]: Pre-relase cleanup
- [#695]: `defmt-rtt`: Refactor rtt [3/2]
- [#689]: `defmt-rtt`: Update to critical-section 1.0
- [#692]: `defmt-macros`: Wrap const fn in const item to ensure compile-time-evaluation.
- [#690]: Satisfy clippy
- [#688]: Release `defmt-decoder 0.3.3`
- [#687]: `CI`: Re-enable `qemu-snapshot (nightly)` tests
- [#686]: `CI`: Temporarily disable `qemu-snapshot (nightly)`
- [#684]: Fix `syn` dependency version in `defmt-macros`.
- [#683]: `defmt-rtt`: Make sure the whole RTT structure is in RAM
- [#682]: `defmt-print`: exit when stdin is closed
- [#681]: Make use of i/o locking being static since rust `1.61`.
- [#679]: `CI`: Add changelog enforcer
- [#678]: Satisfy clippy

[#703]: https://github.com/knurling-rs/defmt/pull/703
[#701]: https://github.com/knurling-rs/defmt/pull/701
[#695]: https://github.com/knurling-rs/defmt/pull/695
[#692]: https://github.com/knurling-rs/defmt/pull/692
[#690]: https://github.com/knurling-rs/defmt/pull/690
[#688]: https://github.com/knurling-rs/defmt/pull/688
[#687]: https://github.com/knurling-rs/defmt/pull/687
[#686]: https://github.com/knurling-rs/defmt/pull/686
[#684]: https://github.com/knurling-rs/defmt/pull/684
[#683]: https://github.com/knurling-rs/defmt/pull/683
[#682]: https://github.com/knurling-rs/defmt/pull/682
[#681]: https://github.com/knurling-rs/defmt/pull/681
[#679]: https://github.com/knurling-rs/defmt/pull/679
[#678]: https://github.com/knurling-rs/defmt/pull/678

## [0.3.2] - 2022-05-31

- [#675]: Release `defmt 0.3.2` and fix `defmt-macros`-releated compile-error
- [#669]: Refine docs for `--json` flag

## [0.3.1] - 2022-03-10

### Added

- [#662]: `#[derive(Format)]` now accepts attribute fields to format fields using the `Debug2Format` adapter instead of a `Format` implementation.
- [#661]: Add tests for Cell types
- [#656]: Implement `defmt::Format` for `Cell` and `RefCell`
- [#630]: Add test instructions to README.md

[#662]: https://github.com/knurling-rs/defmt/pull/662
[#661]: https://github.com/knurling-rs/defmt/pull/661
[#656]: https://github.com/knurling-rs/defmt/pull/656
[#630]: https://github.com/knurling-rs/defmt/pull/630

### Changed

- [#659]: mark extern::acquire() and extern::release() as unsafe. this is not a breaking change; this is an internal API
- [#640]: use crate [critical-section](https://crates.io/crates/critical-section) in defmt-rtt
- [#634]: update ELF parsing dependencies
- [#633]: make RTT buffer size configurable
- [#626]: Make errror message more meaningful in case of version-mismatch

[#659]: https://github.com/knurling-rs/defmt/pull/659
[#640]: https://github.com/knurling-rs/defmt/pull/640
[#634]: https://github.com/knurling-rs/defmt/pull/634
[#633]: https://github.com/knurling-rs/defmt/pull/633
[#626]: https://github.com/knurling-rs/defmt/pull/626

## [v0.3.0] - 2021-11-09

- [#618]: Support #[ignore] attribute in defmt_test
- [#621]: Readme Diagram: Replace duplicate defmt-itm with defmt-rtt
- [#617]: Add display hint to output `u64` as ISO8601 time
- [#620]: Tidy up: remove unused code and dependencies
- [#619]: Update all crates to rust edition 2021! 🎉
- [#610]: `defmt-print`: Log if malformed frame gets skipped
- [#547]: Migration guide `v0.2.x` to `v0.3.0`
- [#604]: defmt-test: `#[cfg(test)]` the `#[defmt_test::tests]` module
- [#616]: Update user guide part of the book
- [#615]: Document how to deal with backward compatibility breakage
- [#614]: Bugfix: decoder breaks with pipe symbol
- [#605]: Properly handle the `off` pseudo-level in presence of nested logging directives
- [#611]: Fix `cargo doc`-warnings
- [#608]: `decoder`: Fix that `defmt::println!` shows leading space when timestamps are disabled
- [#601]: Move crate `defmt` to `defmt/`
- [#600]: Run snapshot & backcompat tests in dev mode only
- [#519]: Target-side `env_logger`-like env filter
- [#592]: xtask: add backward compability test
- [#598]: `defmt-print`: Recover from decoding-errors
- [#569]: Add defmt `println!` macro
- [#580]: Structure `defmt::export`
- [#594]: panic-probe: use UDF instruction on nested panics
- [#591]: Remove timestamps from snapshot test
- [#585]: Add xtask option to run a single snapshot test by name
- [#589]: Implement `Format` for arrays of any length
- [#587]: Tweak inline attributes to remove machine code duplication
- [#574]: Refactor rtt [1/2]
- [#584]: Remove outdated doc "you may only call write! once"
- [#582]: Release of `panic-probe v0.2.1`
- [#581]: Add impl for `alloc::Layout`
- [#579]: defmt-rtt: fix check for blocking RTT
- [#577]: Fix typo in `cfg` of encoding-feature-`compile_error!`
- [#578]: `qemu`: Allow dead code
- [#550]: `defmt::flush()`
- [#572]: `defmt-decoder`: `impl TryFrom<Option<String>> for Encoding`
- [#570]: Support referring to `Self` in bitflags constants
- [#568]: Encoding docs.
- [#564]: Make order of bitflags values deterministic
- [#561]: Remove unused cortex-m-rt in panic-probe a=Dirbaio
- [#562]: Remove call to fill in user survey from `README`
- [#560]: Update cortex-m-rt crate from `0.6` to `0.7`
- [#557]: Add impl for TryFromSliceError
- [#556]: Add impl for TryFromIntError
- [#551]: Display git version & date to introduction section
- [#540]: Separate "crate version" from "wire format version
- [#549]: Fix clippy warnings.
- [#545]: Revert "`build.rs`: Obtain version from macro; simplify"
- [#539]: Add optional rzCOBS encoding+framing
- [#518]: `build.rs`: Obtain version from macro; simplify
- [#543]: `CI`: Temporarily drop backward-compatibility check
- [#542]: `snapshot-tests`: Test alternate hint for bitfields
- [#538]: Fix wrong bit count in comment.
- [#537]: `snapshot-tests`: Delete `:?` hint without impact
- [#531]: refactor the `macros` crate
- [#534]: Attribute test progress message to the test in question;
- [#535]: Don't print leading space when timestamp is absent
- [#529]: Refactor user-guide of `book`
- [#528]: Support bitflags
- [#533]: Adds add for user survey into readme.
- [#527]: `book`: Add logo and support text to introduction
- [#526]: `decoder`: Simplify tests
- [#359]: Implement precedence of inner display hint
- [#523]: Minimize dependencies
- [#508]: [5/n] Format trait v2
- [#522]: Replace `µs` hint with `us`
- [#521]: [3/n] Remove u24
- [#516]: `xtask`: Only install additional targets for tests that require them
- [#512]: Add overwrite option for xtask cross results.
- [#514]: extend raw pointer implementation to include !Format types
- [#513]: book/duplicates.md: discriminator -> disambiguator
- [#507]: [2/n] - Remove code-size-costly optimizations
- [#505]: [1/n] - Logger trait v2.

[#618]: https://github.com/knurling-rs/defmt/pull/618
[#621]: https://github.com/knurling-rs/defmt/pull/621
[#617]: https://github.com/knurling-rs/defmt/pull/617
[#620]: https://github.com/knurling-rs/defmt/pull/620
[#619]: https://github.com/knurling-rs/defmt/pull/619
[#610]: https://github.com/knurling-rs/defmt/pull/610
[#547]: https://github.com/knurling-rs/defmt/pull/547
[#604]: https://github.com/knurling-rs/defmt/pull/604
[#616]: https://github.com/knurling-rs/defmt/pull/616
[#615]: https://github.com/knurling-rs/defmt/pull/615
[#614]: https://github.com/knurling-rs/defmt/pull/614
[#605]: https://github.com/knurling-rs/defmt/pull/605
[#611]: https://github.com/knurling-rs/defmt/pull/611
[#608]: https://github.com/knurling-rs/defmt/pull/608
[#601]: https://github.com/knurling-rs/defmt/pull/601
[#600]: https://github.com/knurling-rs/defmt/pull/600
[#519]: https://github.com/knurling-rs/defmt/pull/519
[#592]: https://github.com/knurling-rs/defmt/pull/592
[#598]: https://github.com/knurling-rs/defmt/pull/598
[#569]: https://github.com/knurling-rs/defmt/pull/569
[#580]: https://github.com/knurling-rs/defmt/pull/580
[#594]: https://github.com/knurling-rs/defmt/pull/594
[#591]: https://github.com/knurling-rs/defmt/pull/591
[#585]: https://github.com/knurling-rs/defmt/pull/585
[#589]: https://github.com/knurling-rs/defmt/pull/589
[#587]: https://github.com/knurling-rs/defmt/pull/587
[#574]: https://github.com/knurling-rs/defmt/pull/574
[#584]: https://github.com/knurling-rs/defmt/pull/584
[#582]: https://github.com/knurling-rs/defmt/pull/582
[#581]: https://github.com/knurling-rs/defmt/pull/581
[#579]: https://github.com/knurling-rs/defmt/pull/579
[#577]: https://github.com/knurling-rs/defmt/pull/577
[#578]: https://github.com/knurling-rs/defmt/pull/578
[#550]: https://github.com/knurling-rs/defmt/pull/550
[#572]: https://github.com/knurling-rs/defmt/pull/572
[#570]: https://github.com/knurling-rs/defmt/pull/570
[#568]: https://github.com/knurling-rs/defmt/pull/568
[#564]: https://github.com/knurling-rs/defmt/pull/564
[#561]: https://github.com/knurling-rs/defmt/pull/561
[#562]: https://github.com/knurling-rs/defmt/pull/562
[#560]: https://github.com/knurling-rs/defmt/pull/560
[#557]: https://github.com/knurling-rs/defmt/pull/557
[#556]: https://github.com/knurling-rs/defmt/pull/556
[#551]: https://github.com/knurling-rs/defmt/pull/551
[#540]: https://github.com/knurling-rs/defmt/pull/540
[#549]: https://github.com/knurling-rs/defmt/pull/549
[#545]: https://github.com/knurling-rs/defmt/pull/545
[#539]: https://github.com/knurling-rs/defmt/pull/539
[#518]: https://github.com/knurling-rs/defmt/pull/518
[#543]: https://github.com/knurling-rs/defmt/pull/543
[#542]: https://github.com/knurling-rs/defmt/pull/542
[#538]: https://github.com/knurling-rs/defmt/pull/538
[#537]: https://github.com/knurling-rs/defmt/pull/537
[#534]: https://github.com/knurling-rs/defmt/pull/534
[#531]: https://github.com/knurling-rs/defmt/pull/531
[#535]: https://github.com/knurling-rs/defmt/pull/535
[#529]: https://github.com/knurling-rs/defmt/pull/529
[#528]: https://github.com/knurling-rs/defmt/pull/528
[#533]: https://github.com/knurling-rs/defmt/pull/533
[#527]: https://github.com/knurling-rs/defmt/pull/527
[#526]: https://github.com/knurling-rs/defmt/pull/526
[#359]: https://github.com/knurling-rs/defmt/pull/359
[#523]: https://github.com/knurling-rs/defmt/pull/523
[#508]: https://github.com/knurling-rs/defmt/pull/508
[#522]: https://github.com/knurling-rs/defmt/pull/522
[#521]: https://github.com/knurling-rs/defmt/pull/521
[#516]: https://github.com/knurling-rs/defmt/pull/516
[#512]: https://github.com/knurling-rs/defmt/pull/512
[#514]: https://github.com/knurling-rs/defmt/pull/514
[#513]: https://github.com/knurling-rs/defmt/pull/513
[#507]: https://github.com/knurling-rs/defmt/pull/507
[#505]: https://github.com/knurling-rs/defmt/pull/505

## [v0.2.3] - 2021-06-17

### Added

- [#499] Illustrate structure of the defmt crates
- [#503] Add alternate hint ('#')
- [#509] `impl Format for NonZero*`

### Changed

- [#488] Structure `impl Format`s into multiple files
- [#496] Bump build-dep `semver` to  `1.0`
- [#489] Structure lib
- [#500] book: fix leftover old formatting syntax; typos
- [#510] `CI`: Don't install MacOS dependency which is included by default

### Fixed

- [#497] `macros`: match unused vars if logging is disabled

[#488]: https://github.com/knurling-rs/defmt/pull/488
[#496]: https://github.com/knurling-rs/defmt/pull/496
[#497]: https://github.com/knurling-rs/defmt/pull/497
[#489]: https://github.com/knurling-rs/defmt/pull/489
[#499]: https://github.com/knurling-rs/defmt/pull/499
[#500]: https://github.com/knurling-rs/defmt/pull/500
[#503]: https://github.com/knurling-rs/defmt/pull/503
[#509]: https://github.com/knurling-rs/defmt/pull/509
[#510]: https://github.com/knurling-rs/defmt/pull/510

## [v0.2.2] - 2021-05-20

### Added

- [#446] Add usage examples for `Debug2Format`, `Display2Format`
- [#464] `impl<T> Format for {*const, *mut} T where T: Format + ?Sized`
- [#472] `impl Format for` the core::{iter, ops, slice} structs
- [#473] `impl Format for` all the `Cow`s
- [#478] add `dbg!` macro

### Changed

- [#477] Disable logging calls via conditional compilation when all defmt features are disabled

[#446]: https://github.com/knurling-rs/defmt/pull/446
[#464]: https://github.com/knurling-rs/defmt/pull/464
[#472]: https://github.com/knurling-rs/defmt/pull/472
[#473]: https://github.com/knurling-rs/defmt/pull/473
[#477]: https://github.com/knurling-rs/defmt/pull/477
[#478]: https://github.com/knurling-rs/defmt/pull/478

## [v0.2.1] - 2021-03-08

### Added

- [#403] Add knurling logo to API docs

### Fixed

- [#413] Fix docs-rs build, by disabling feature "unstable-test"
- [#427] Drop outdated note about `defmt v0.2.0` from book

[#403]: https://github.com/knurling-rs/defmt/pull/403
[#413]: https://github.com/knurling-rs/defmt/pull/413
[#427]: https://github.com/knurling-rs/defmt/pull/427

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

[Unreleased]: https://github.com/knurling-rs/defmt/compare/defmt-v0.3.2...main
[v0.3.2]: https://github.com/knurling-rs/defmt/compare/defmt-v0.3.1...defmt-v0.3.2
[v0.3.1]: https://github.com/knurling-rs/defmt/compare/defmt-v0.3.0...defmt-v0.3.1
[v0.3.0]: https://github.com/knurling-rs/defmt/compare/defmt-v0.2.3...defmt-v0.3.0
[v0.2.3]: https://github.com/knurling-rs/defmt/compare/defmt-v0.2.2...defmt-v0.2.3
[v0.2.2]: https://github.com/knurling-rs/defmt/compare/defmt-v0.2.1...defmt-v0.2.2
[v0.2.1]: https://github.com/knurling-rs/defmt/compare/defmt-v0.2.0...defmt-v0.2.1
[v0.2.0]: https://github.com/knurling-rs/defmt/compare/defmt-v0.1.3...defmt-v0.2.0
[v0.1.3]: https://github.com/knurling-rs/defmt/compare/defmt-v0.1.2...defmt-v0.1.3
[v0.1.2]: https://github.com/knurling-rs/defmt/compare/v0.1.1...defmt-v0.1.2
[v0.1.1]: https://github.com/knurling-rs/defmt/compare/v0.1.0...v0.1.1
