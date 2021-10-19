# Migrating from `v0.2.x` to `v0.3.0`

This guide covers how to upgrade a library or application using `defmt v0.2.x` to version `v0.3.0`.

## `Cargo.toml`

Update the version of `defmt` to `"0.3"` (or `"0.3.0"`, which is equivalent).

Additionally please remove the `defmt-*` cargo features from your `[features]` section.

```diff
[dependencies]

- defmt = "0.2"
+ defmt = "0.3"

[features]
default = [
    "other-feature",
-   "defmt-default",
-   "dependency-a/defmt-trace",
]

other-feature = []

- defmt-default = []
- defmt-trace = []
- defmt-debug = []
- defmt-info = []
- defmt-warn = []
- defmt-error = []
```

## Set the log-level with `DEFMT_LOG`

Setting the log-level via cargo features is superseded by the new `DEFMT_LOG` environment variable.

To log everything on the `INFO` level and above, run your application like following:

```console
$ DEFMT_LOG=info cargo run
```

For more details how to configure the log-level using `DEFMT_LOG` see the [user docs](./filtering.md#defmt_log).

This new mechanism is modelled to be similar to the well-known `RUST_LOG` and now also supports log-configuration down to the module-level!

## Upgrade display hints

> 💡 Easily change the display hints, using the search-and-replace feature of your editor. *([vs code](https://code.visualstudio.com/docs/editor/codebasics#_search-and-replace))*

### Rename display hint `µs` to `us`

Due to ambiguity in-between `µ` (micro sign) and `μ` (small mu), the display hint for microseconds changed to be `us`.

Therefore you likely need to update your timestamp definition.

```diff
- defmt::timestamp!("{=u32:µs}", {
+ defmt::timestamp!("{=u32:us}", {
    // ...
});
```

As well as all other logging calls where you were using `µs`.

```diff
- defmt::info!("{=u8:µs}", time)
+ defmt::info!("{=u8:us}", time)
```

### Drop `u24` type hint

The `u24` type hint got dropped, cause it was confusing users and complicated internal parser logic.

Therefore replace it with `u32` in all logging calls.

```diff
- defmt::info!("{=u24}", 42);
+ defmt::info!("{=u32}", 42);
```

## Adapt manual `trait Logger` implementations

The `Logger` trait has seen a couple of big changes, for one the function signatures of a few methods have changed, the previous `Write` trait got removed while its `write` method is part of `Logger` now and the method `flush` was added.

> 💡 If you are using one of our loggers, `defmt-rtt` and `defmt-itm`, no action is required!

Let's see what a new implementation of the `Logger` in crate `defmt-itm` looks like compared to the previous implementation in version `0.2`. The following abbreviated example code shows how the `Logger` worked before.

```rust,ignore
#[defmt::global_logger]
struct Logger;

static TAKEN: AtomicBool = AtomicBool::new(false);

unsafe impl defmt::Logger for Logger {
    fn acquire() -> Option<NonNull<dyn defmt::Write>> {
        // disable interrupts

        if !TAKEN.load(Ordering::Relaxed) {
            // acquire the lock
            TAKEN.store(true, Ordering::Relaxed);
            Some(NonNull::from(&Logger as &dyn defmt::Write))
        } else {
            None
        }
    }

    unsafe fn release(_: NonNull<dyn defmt::Write>) {
        // release the lock
        TAKEN.store(false, Ordering::Relaxed);

        // re-enable interrupts
    }
}

impl defmt::Write for Logger {
    fn write(&mut self, bytes: &[u8]) {
        unsafe { itm::write_all(&mut (*ITM::ptr()).stim[0], bytes) }
    }
}
```

And here is how it conceptually works now:

```rust
# extern crate defmt;
# use std::sync::atomic::{AtomicBool,Ordering};

#[defmt::global_logger]
struct Logger;

static TAKEN: AtomicBool = AtomicBool::new(false);
static mut ENCODER: defmt::Encoder = defmt::Encoder::new();

unsafe impl defmt::Logger for Logger {
    fn acquire() {
        // disable interrupts

        if TAKEN.load(Ordering::Relaxed) {
            panic!("defmt logger taken reentrantly")
        }

        // acquire the lock
        TAKEN.store(true, Ordering::Relaxed);

        unsafe { ENCODER.start_frame(do_write) }
    }

    unsafe fn flush() {
        // Omitted. We will come to this in a second.
    }

    unsafe fn release() {
        ENCODER.end_frame(do_write);

        // release the lock
        TAKEN.store(false, Ordering::Relaxed);

        // re-enable interrupts
    }

    unsafe fn write(bytes: &[u8]) {
        ENCODER.write(bytes, do_write);
    }
}

fn do_write(bytes: &[u8]) {
    unsafe { itm::write_all(&mut (*ITM::ptr()).stim[0], bytes) }
}

# // mock cortex_m-crate
# mod itm {
#     pub unsafe fn write_all(a: &mut u8, b: &[u8]) {}
# }
# struct ITM { stim: [u8; 1] }
# impl ITM {
#     fn ptr() -> *mut ITM { &mut ITM { stim: [0] } as *mut _ }
# }
```

Let us go through the changes step-by-step:
- Drop `trait Write`:
  - Extract the `fn write` from the `trait Write` and name it `fn do_write`.
  - Remove the `impl defmt::Write for Logger`.
  - Remove the the first argument of `&mut self` from `fn do_write`.
- Add a new `static mut ENCODER: defmt::Encoder = defmt::Encoder::new()` outside the `impl defmt::Logger for Logger`-block.
- Adapt `fn acquire`:
  - Remove the return type `Option<NonNull<dyn defmt::Write>>` from `fn acquire`.
  - Replace all `return None` with an explicit `panic!`, with a descriptive error message.
  - Call `unsafe { ENCODER.start_frame(do_write) }`, after you acquired the lock.
- Adapt `fn release`:
  - Call `ENCODER.end_frame(do_write);`, before releasing the lock.
- Add new method `unsafe fn write` to `impl defmt::Logger for Logger`:
    ```rust,noplayground
    # extern crate defmt;
    # static mut ENCODER: defmt::Encoder = defmt::Encoder::new();
    # fn do_write(bytes: &[u8]) {}
    unsafe fn write(bytes: &[u8]) {
        ENCODER.write(bytes, do_write);
    }
    ```

And that is already it!

### Flush

One final thing is left before your custom `trait Logger` implementation works again: You need to implement `fn flush`.

This functionality is what gets used when calling `defmt::flush`, [whose docs say](https://docs.rs/defmt/*/defmt/fn.flush.html):
> Block until host has read all pending data.
>
> The flush operation will not fail, but might not succeed in flushing all pending data. It is implemented as a “best effort” operation.

The idea is to ensure that _all data_ is read by the host. Take `defmt-rtt` as an example:

```rust
# extern crate defmt;
# use std::sync::atomic::{AtomicUsize,Ordering};

# #[defmt::global_logger]
# struct Logger;

# static READ: AtomicUsize = AtomicUsize::new(0);
# static WRITE: AtomicUsize = AtomicUsize::new(0);

unsafe impl defmt::Logger for Logger {
    fn acquire() {
        // ...
    }

    unsafe fn flush() {
        // busy wait, until the read- catches up with the write-pointer
        let read = || READ.load(Ordering::Relaxed);
        let write = || WRITE.load(Ordering::Relaxed);
        while read() != write() {}
    }

    unsafe fn release() {
        // ...
    }

    unsafe fn write(bytes: &[u8]) {
        // ...
    }
}
```

If your transport doesn't allow to ensure that _all data_ got read, `flush` should at least flush _as much data as possible_. Take `defmt-itm` as an example for this:

```rust
# extern crate defmt;

# #[defmt::global_logger]
# struct Logger;

unsafe impl defmt::Logger for Logger {
    fn acquire() {
        // ...
    }

    unsafe fn flush() {
        // wait for the queue to be able to accept more data
        while !stim_0().is_fifo_ready() {}

        // delay "a bit" to drain the queue
        // This is a heuristic and might be too short in reality.
        // Please open an issue if it is!
        asm::delay(100);
    }

    unsafe fn release() {
        // ...
    }

    unsafe fn write(bytes: &[u8]) {
        // ...
    }
}

# // mock cortex_m-crate
# struct STIM0;
# impl STIM0 {
#     fn is_fifo_ready(&self) -> bool { true }
# }
# fn stim_0() -> STIM0 { STIM0 }
# mod asm {
#     pub fn delay(cycles: usize) {}
# }
```

`defmt::flush` can be used before a hard-reset of the device, where you would loose data if you do not flush.

Since you are the expert of your transport, implement the method now!

## Unified `probe-run` backtrace options

The new `--backtrace` and `--backtrace-limit` of `probe-run` should simplify the configuration.

```console
cargo run --bin panic --backtrace=always --backtrace-limit=5
```

Using `--backtrace` you can now specify if you want to have a backtrace `never`, `always` or only in case of an exception (the `auto` option, which is the default). For the latter two options you can specify the maximum backtrace length, which defaults to `50` and can be set to unlimited with `0`.

See [the `probe-run`-README](https://github.com/knurling-rs/probe-run#backtrace-options) for all the options.

## Congrats! 🎉

If you followed all the steps in this guide, your application should work flawlessly again and make use of all the internal and user-facing improvements shipped in this release! 
