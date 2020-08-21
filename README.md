# `defmt`

> Efficient, deferred formatting for logging on embedded systems

> **ALPHA PREVIEW** `defmt` wire format has not been finalized yet. When
> using the framework make sure you use the *same* "version" (commit hash) for
> all components (target side and host side).

`defmt` ("de format", short for "deferred formatting") is a highly efficient logging framework that targets resource-constrained devices, like microcontrollers.

The fastest way to get started with `defmt` is to follow [this blog post] to set up a Cortex-M embedded project.

[this blog post]: https://ferrous-systems.com/blog/defmt

For more details about the framework check the book at https://defmt.ferrous-systems.com

## Support

`probe-run` is part of the [Knurling] project, [Ferrous Systems]' effort at
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
