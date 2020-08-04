# #[timestamp]

*Applications* that, directly or transitively, use any of `binfmt` logging macros need to define a `#[timestamp]` function or include one in their dependency graph.

All logs are timestamped.
The `#[timestamp]` function specifies how the timestamp is computed.
This function must have signature `fn() -> u64` and on each invocation *should* return a non-decreasing value.
The function is not `unsafe` meaning that it must be thread-safe and interrupt-safe.

## No timestamp

To omit timestamp information use this `#[timestamp]` function:

``` rust
#[binfmt::timestamp]
fn timestamp() -> u64 {
    0
}
```

## Atomic timestamp

A simple `timestamp` function that does not depend on device specific features and it's good enough for development is shown below:

``` rust
// WARNING may overflow and wrap-around in long lived apps
#[binfmt::timestamp]
fn timestamp() -> u64 {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    COUNT.fetch_add(1, Ordering::Relaxed) as u64
}
```

## Hardware timestamp

A `timestamp` function that uses a device-specific monotonic timer can directly read a MMIO register.
It's OK if the function returns `0` while the timer is disabled.

``` rust
// WARNING may overflow and wrap-around in long lived apps
#[binfmt::timestamp]
fn timestamp() -> u64 {
    // NOTE(interrupt-safe) single instruction volatile read operation
    unsafe { monotonic_timer_counter_register().read_volatile() as u64 }
}

fn main() -> ! {
    info!(..); // timestamp = 0
    debug!(..); // timestamp = 0
    enable_monotonic_counter();
    info!(..); // timestamp >= 0
}
```

### 64-bit extension

Microcontrollers usually have only 32-bit counters.
Some of them may provide functionality to make one 32-bit counter increase the count of a second 32-bit counter when the first wrap arounds.
Where that functionality is not available, a 64-bit counter can be emulated using interrupts:

``` rust
static OVERFLOW_COUNT: AtomicU32 = AtomicU32::new(0);

// NOTE must run at highest priority
#[interrupt]
fn on_first_counter_overflow() {
    let ord = Ordering::Relaxed;
    OVERFLOW_COUNT.store(OVERFLOW_COUNT.load(ord) + 1, ord);
}
```

To read the 64-bit value in a lock-free manner the following algorithm can be used (pseudo-code):

``` text
do {
  high1 <- read_high_count()
  low <- read_low_count()
  high2 <- read_high_count()
} while (high1 != high2)
count: u64 <- (high1 << 32) | low
```

The loop should be kept as tight as possible and the read operations must be single-instruction operations.
