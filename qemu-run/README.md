# `qemu-run`

> Runs [`qemu-system-arm`] but decodes [`defmt`] data sent to semihosting

[`qemu-system-arm`]: https://www.qemu.org/docs/master/system/target-arm.html
[`defmt`]: https://crates.io/crates/defmt

## Using

Set as your cargo runner, e.g. in your `.cargo/config.toml` file:

```toml
[target.thumbv7em-none-eabihf]
runner = "qemu-run -machine lm3s6965evb"
```

It will execute `qemu-system-arm`, pass the given `-machine` argument, pass
additional arguments to configure semihosting, and pipe semihosting data into
`defmt-decoder` to be decoded and printed to the console.

## Support

`defmt-print` is part of the [Knurling] project, [Ferrous Systems]' effort at
improving tooling used to develop for embedded systems.

If you think that our work is useful, consider sponsoring it via [GitHub
Sponsors].

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)

- MIT license ([LICENSE-MIT](../LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as above, without any additional terms or conditions.

[Knurling]: https://knurling.ferrous-systems.com/
[Ferrous Systems]: https://ferrous-systems.com/
[GitHub Sponsors]: https://github.com/sponsors/knurling-rs
