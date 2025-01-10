# `defmt-semihosting`

> Transmit [`defmt`] log messages over the Cortex-M Debugger Semihosting protocol.

[`defmt`]: https://github.com/knurling-rs/defmt

`defmt` ("de format", short for "deferred formatting") is a highly efficient logging framework that targets resource-constrained devices, like microcontrollers.

For more details about the framework check the book at <https://defmt.ferrous-systems.com>.

Note that Semihosting operations are very slow. This is really only useful if
you are running in QEMU can hence can't use RTT. See
<https://github.com/ferrous-systems/rust-training/tree/9d5f48c9a62ccadb11d942847de47146b15638d0/example-code/qemu-thumbv7em>
for an example of using this crate with QEMU.

## Support

`defmt-semihosting` is part of the [Knurling] project, [Ferrous Systems]' effort at
improving tooling used to develop for embedded systems.

If you think that our work is useful, consider sponsoring it via [GitHub
Sponsors].

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)

- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as above, without any additional terms or conditions.

[Knurling]: https://knurling.ferrous-systems.com/
[Ferrous Systems]: https://ferrous-systems.com/
[GitHub Sponsors]: https://github.com/sponsors/knurling-rs
