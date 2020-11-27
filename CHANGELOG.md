# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

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

[Unreleased]: https://github.com/knurling-rs/defmt/compare/defmt-v0.1.2...main
[v0.1.1]: https://github.com/knurling-rs/defmt/compare/v0.1.0...v0.1.1
[v0.1.2]: https://github.com/knurling-rs/defmt/compare/v0.1.1...defmt-v0.1.2
