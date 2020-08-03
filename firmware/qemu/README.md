# `firmware`

Firmware configured to run on QEMU to try out defmt end-to-end (encoder + decoder + singleton) on an emulated microcontroller

## dependencies
- [qemu](https://www.qemu.org/download/)

## running

Run the examples with:

``` console
$ # alias for cargo-run --bin as set in .cargo/config
$ cargo rb log
(...)
0.000000 INFO Hello!
0.000001 INFO World!
(...)
```

or
``` console
$ # alias for cargo-run --release --bin as set in .cargo/config
$ cargo rrb log
(...)
0.000000 INFO Hello!
0.000001 INFO World!
(...)
```

Note the difference in log levels for debug and release; for details see the [Logging level filtering](../README.md#logging-level-filtering) documentation.
