# `defmt-stdout`

> Transmit [`defmt`] log messages to stdout or a file on operating systems with `std` support

[`defmt`]: https://crates.io/crates/defmt

`defmt-stdout` is a `defmt` global logger for programs that run on a host operating system
(Linux, macOS, ...) rather than on bare metal. This is useful for running unit tests or host
builds of code that logs via `defmt`.

The binary defmt wire data is written to stdout by default. Set the `DEFMT_STDOUT_FILE`
environment variable to write it to a file instead. Decode the output with [`defmt-print`]:

```console
$ cargo run | defmt-print -e target/debug/my-app
```

[`defmt-print`]: https://crates.io/crates/defmt-print

## Support

`defmt-stdout` is part of the [Knurling] project, [Ferrous Systems]' effort at
improving tooling used to develop for embedded systems.

If you think that our work is useful, consider sponsoring it via [GitHub
Sponsors].

[Knurling]: https://knurling.ferrous-systems.com/
[Ferrous Systems]: https://ferrous-systems.com/
[GitHub Sponsors]: https://github.com/sponsors/knurling-rs

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as above, without any additional terms or conditions.
