# #[global_logger]

> *Applications* that, directly or transitively, use any of `defmt` logging macros need to define a `#[global_logger]` or include one in their dependency graph.

This is similar to how the `alloc` crate depends on a `#[global_allocator]`.

The `global_logger` defines how data is moved from the *device*, where the application runs, to the host, where logs will be formatted and displayed.
`global_logger` is transport agnostic: you can use a serial interface, serial over USB, RTT, semihosting, Ethernet, 6LoWPAN, etc. to transfer the data.

The `global_logger` interface comprises the trait `Logger` and the `#[global_logger]` attribute.

## The `Logger` trait

Firstly `Logger` specifies how to acquire and release a handle to a global logger, as well as how the data is put on the wire.

```rust
# extern crate defmt;
# struct Logger;
#
unsafe impl defmt::Logger for Logger {
    fn acquire() {
        // ...
    }
    unsafe fn release() {
        // ...
    }
    unsafe fn write(bytes: &[u8]) {
        // ...
    }
}
```

The `write` method is not allowed to fail.
Buffering, rather than waiting on I/O, is recommended.
If using buffering `write` should not overwrite old data as this can corrupt log frames and most printers cannot deal with incomplete log frames.

See the API documentation for more details about the safety requirements of the acquire-release mechanism.


## The `#[global_logger]` attribute

Secondly, `#[global_logger]` specifies which `Logger` implementation will be used by the application.

`#[global_logger]` must be used on a *unit* struct, a struct with no fields, which must implement the `Logger` trait.
It's recommended that this struct is kept private.

```rust
# extern crate defmt;
#
#[defmt::global_logger]
struct Logger;

unsafe impl defmt::Logger for Logger {
    // ...
    # fn acquire() {}
    # unsafe fn release() {}
    # unsafe fn write(bytes: &[u8]) {}
}
```

> ⚠️ Only a single `#[global_logger]` struct can appear in the dependency graph of an application.
>
> Therefore the `global_logger` should be selected *at the top* of the dependency graph, that is in the application crate.

There are two general ways to implement a `global_logger`.

## Single logging channel

The first form uses a single channel.
This means that all execution contexts (i.e. threads OR `main` + interrupt handlers) use the same logging channel.
In an application that uses interrupts this means that `acquire` must disable interrupts and `release` must re-enable interrupts.
This synchronizes access to the single channel, from contexts running at different priority levels.

`defmt-semihosting` is an example of this single logging channel approach.

## Multiple logging channels

The other approach uses multiple logging channels: e.g. one for each priority level in an application that uses interrupts.
With this approach logging can be made lock-free: interrupts are not disabled while logging data.
This approach requires channel multiplexing in the transport layer.
RTT, for example, natively supports multiple channels so this is not an issue, but other transports, like ITM, will require that each log frame to be tagged with the channel it belongs to (e.g. one logging channel = ITM channel).

The trade-offs of using more channels are:
- Lock-freedom
- higher memory usage on the target, for buffering
- lower overall throughput, as either different channels need to be polled from the host or the log frames need to be tagged with the channel they belong to
