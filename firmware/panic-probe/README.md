# `panic-probe`

> Panic handler that exits [`probe-rs`] with an error code using semihosting::process::abort.

[`probe-rs`]: https://github.com/probe-rs/probe-rs

`panic-probe` can optionally log the panic message. Enabled one of the following features for that:
* `print-defmt` to print via `defmt::error!(..)`
* `print-log` to print via `log::error!(..)`
* `print-rtt` to print via `rtt_target::rprintln(..)`


 using the [`defmt`] logging framework.
This functionality can be enabled through the `print-defmt` Cargo feature.

[`defmt`]: https://github.com/knurling-rs/defmt

## Support

`panic-probe` is part of the [Knurling] project, [Ferrous Systems]' effort at
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

[Knurling]: https://knurling.ferrous-systems.com
[Ferrous Systems]: https://ferrous-systems.com/
[GitHub Sponsors]: https://github.com/sponsors/knurling-rs
