# `firmware`

Firmware configured to run on QEMU to try out defmt end-to-end (encoder + decoder + singleton) on an emulated microcontroller

## dependencies
- [qemu](https://www.qemu.org/download/)

## running

Run the examples with:

``` console
$ # alias for cargo-run --bin
$ cargo rb log
(..)

$ # alias for cargo-run --release --bin
$ cargo rrb log
(..)
```
