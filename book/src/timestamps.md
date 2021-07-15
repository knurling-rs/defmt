# Timestamps

> *Applications* that, directly or transitively, use any of `defmt` logging macros may use the `timestamp!` macro to define additional data to be included in every log frame.

The `timestamp!` macro may only be used once throughout the crate graph. Its syntax is the same as for the other logging macros, except that `timestamp!` is global and so cannot access any local variables.

By default, no timestamp is provided or transferred over the defmt sink.

## Atomic timestamp

A simple `timestamp` function that does not depend on device specific features and is good enough for development is shown below:

``` rust
# extern crate defmt;
# use std::sync::atomic::{AtomicUsize, Ordering};
// WARNING may overflow and wrap-around in long lived apps
static COUNT: AtomicUsize = AtomicUsize::new(0);
defmt::timestamp!("{=usize}", COUNT.fetch_add(1, Ordering::Relaxed));
```

## Hardware timestamp

A `timestamp` function that uses a device-specific monotonic timer can directly read a MMIO register.
It's OK if the function returns `0` while the timer is disabled.

The `us` display hint can be used to format an integer value as a time in microseconds (eg. `1_000_000` may be displayed as `1.000000`).

``` rust
# extern crate defmt;
# fn monotonic_timer_counter_register() -> *mut u32 {
#     static mut X: u32 = 0;
#     unsafe { &mut X as *mut u32 }
# }
// WARNING may overflow and wrap-around in long lived apps
defmt::timestamp!("{=u32:us}", {
    // NOTE(interrupt-safe) single instruction volatile read operation
    unsafe { monotonic_timer_counter_register().read_volatile() }
});

# fn enable_monotonic_counter() {}
fn main() {
    defmt::info!("..");  // timestamp = 0
    defmt::debug!(".."); // timestamp = 0

    enable_monotonic_counter();
    defmt::info!("..");  // timestamp >= 0
    // ..
}
```

### 64-bit extension

Microcontrollers usually have only 32-bit counters / timers.
Some of them may provide functionality to make one 32-bit counter increase the count of a second 32-bit counter when the first one wraps around.
Where that functionality is not available, a 64-bit counter can be emulated using interrupts:

``` rust
# use std::sync::atomic::{AtomicU32, Ordering};

// the hardware counter is the "low (32-bit) counter"

// this atomic variable is the "high (32-bit) counter"
static OVERFLOW_COUNT: AtomicU32 = AtomicU32::new(0);

// this is an interrupt handler running at highest priority
fn on_first_counter_overflow() {
    let ord = Ordering::Relaxed;
    OVERFLOW_COUNT.store(OVERFLOW_COUNT.load(ord) + 1, ord);
}
```

To read the 64-bit value in a lock-free manner the following algorithm can be used (pseudo-code):

``` text
do {
  high1: u32 <- read_high_count()
  low : u32 <- read_low_count()
  high2 : u32 <- read_high_count()
} while (high1 != high2)
count: u64 <- (high1 << 32) | low
```

The loop should be kept as tight as possible and the read operations must be single-instruction operations.
