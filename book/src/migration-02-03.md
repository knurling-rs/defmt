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
other-feature = []

- defmt-default = []
- defmt-trace = []
- defmt-debug = []
- defmt-info = []
- defmt-warn = []
- defmt-error = []
```

### Encoding

`defmt` now offers two types of encoding: `rzcobs` and `raw`. You don't need to set this explicitly, since there is a default and both encodings are compatible with the corresponding `defmt-decoder` version.

For more information on the encodings look [here](TODO: add link!).

To set one explicitly you need to select it via the corresponding cargo-feature:

```toml
defmt = { version = "0.3", features = ["encoding-rzcobs"] }

# OR

defmt = { version = "0.3", features = ["encoding-raw"] }
```

## Set the log-level with `DEFMT_LOG`

Setting the log-level via cargo features is superseded by the new `DEFMT_LOG` environment variable.

To log everything on the `INFO` level and above, run your application like following:

```console
$ DEFMT_LOG=info cargo run
```

For more details how to configure the log-level using `DEFMT_LOG` see the [user docs](./filtering.md#defmt_log).

## Rename display hint `µs` to `us`

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

> 💡 Easily fix this, using the global search-and-replace feature of your editor/IDE. *([vs code](https://code.visualstudio.com/docs/editor/codebasics#_search-and-replace))*

## Drop `u24` type hint

The `u24` type hint got dropped, cause it was confusing users and complicates the code.

Therefore replace it with `u32` in all logging calls.

```diff
- defmt::info!("{=u24}", 42);
+ defmt::info!("{=u32}", 42);
```

> 💡 Use the global search-and-replace here as well!

## Adapt manual `trait Logger` implementations

The `trait Logger` gained a big rework. The existing methods changed the function signature, the `trait Write` got removed and there are two new methods. Therefore you need to adapt implementations for your custom loggers.

> 💡 If you are using one of our loggers, `defmt-rtt` and `defmt-itm`, no action is required!

Let us see what needs to be done on the example of `defmt-itm`. Here is how the `trait Logger` implementation conceptually worked before:

```rust,ignore
#[defmt::global_logger]
struct Logger;

static TAKEN: AtomicBool = AtomicBool::new(false);

unsafe impl defmt::Logger for Logger {
    fn acquire() -> Option<NonNull<dyn defmt::Write>> {
        if exception {
            return None;
        }

        if !TAKEN.load(Ordering::Relaxed) {
            TAKEN.store(true, Ordering::Relaxed);

            Some(NonNull::from(&Logger as &dyn defmt::Write))
        } else {
            None
        }
    }

    unsafe fn release(_: NonNull<dyn defmt::Write>) {
        TAKEN.store(false, Ordering::Relaxed);
    }
}

impl defmt::Write for Logger {
    fn write(&mut self, bytes: &[u8]) {
        unsafe { itm::write_all(&mut (*ITM::ptr()).stim[0], bytes) }
    }
}
```

And here is, how it conceptually does now:

```rust
# extern crate defmt;
# use std::sync::atomic::{AtomicBool,Ordering};

#[defmt::global_logger]
struct Logger;

static TAKEN: AtomicBool = AtomicBool::new(false);
static mut ENCODER: defmt::Encoder = defmt::Encoder::new();

unsafe impl defmt::Logger for Logger {
    fn acquire() {
        # let exception = false;
        if exception {
            panic!("something bad happened!")
        }

        if TAKEN.load(Ordering::Relaxed) {
            panic!("defmt logger taken reentrantly")
        }

        // acquire the lock
        TAKEN.store(true, Ordering::Relaxed);

        unsafe { ENCODER.start_frame(do_write) }
    }

    unsafe fn flush() {
        // Omitted. We will come to this in a second
    }

    unsafe fn release() {
        ENCODER.end_frame(do_write);

        // release the lock
        TAKEN.store(false, Ordering::Relaxed);
    }

    unsafe fn write(bytes: &[u8]) {
        ENCODER.write(bytes, do_write);
    }
}

fn do_write(bytes: &[u8]) {
    unsafe { itm::write_all(&mut (*ITM::ptr()).stim[0], bytes) }
}

# mod itm {
#     pub unsafe fn write_all(a: &mut u8, b: &[u8]) {}
# }
# struct ITM { stim: [u8; 1] }
# impl ITM {
#     fn ptr() -> *mut ITM { &mut ITM { stim: [0] } as *mut _ }
# }
```

### Flush

---

TODO

- [ ] `#505`: Logger trait v2
- [x] `#521`: [3/n] Remove u24
- [x] `#522`: Replace `µs` hint with `us`
- [ ] `#508`: [5/n] Format trait v2
  - no Write trait anymore
- [x] `#519`: `DEFMT_LOG`

