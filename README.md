# `binfmt`

## Features

- `println!`-like formatting
- Multiple logging levels: error, info, warn, debug, trace
- Crate-level logging level filters
- Timestamped logs

## Current limitations

- Object format must be ELF
- Custom linking (linker script) is required
- Single, global logger instance
- The `@` character is not allowed in strings but otherwise UTF-8 is supported &#x1F44D
- No x86 support. This architecture is exclusively used for testing at the moment.
- ???

## Intended use

In its current iteration `binfmt` mainly targets tiny embedded devices that have no mean to display information to the developer, e.g. a screen.
In this scenario logs need to be transferred to a second machine, usually a PC/laptop, before they can be displayed to the developer/end-user.
`binfmt` operating principles, however, are applicable to beefier machines and could be use to improve the logging performance of x86 web server applications and the like.

## Operating principle

`binfmt` achieves high performance using deferred formatting and string compression.
Deferred formatting means that formatting is not done on the machine that's logging data but on a second machine.
That is instead of formatting `255u8` into `"255"` and sending the string, the single-byte binary data is sent to a second machine and the formatting happens there.
`binfmt`'s string compression consists of building a table of string literals, like `"Hello, world"` or `"The answer is {:?}"`, at compile time.
At runtime the logging machine sends *indices* instead of complete strings.

## User guide

Unless indicated otherwise these sections apply to the use of `binfmt` in libraries and applications.

### Logging

Logging is done using the `error`, `warn`, `info`, `debug` and `trace` macros.
Each macro logs at the logging level indicated in its name.
The syntax of these macros is roughly the same as the `println` macro.
Positional parameters are supported but named parameters are not.
Escaping rules are the same: the characters `{` and `}` are escaped as `{{` and `}}`.
The biggest different is in the supported formatting parameters (`:?`, `:>4`, `:04`).

``` rust
// -> INFO:  message arrived (length=80)
binfmt::info!(
    "message arrived (length={:?})",
    len /*: usize */,
);

// -> DEBUG: Header { source: 2, destination: 3, sequence: 16 }
binfmt::debug!("{:?}", message.header() /*: Header */);
```

Unlike `core::fmt` which has several formatting traits (`Debug`, `Display`), `binfmt` has a single formatting trait called `Format`.
The `:?` formatting parameter indicates that the `Format` trait will be used.
When `:?` is used the corresponding argument must implement the `Format` trait.

``` rust
binfmt::trace!("{:?}", x);
//                     ^ must implement the `Format` trait
```

### Primitives

In addition to `:?` there are formatting parameters for several primitive types.
These parameters follow the syntax `:<type>`. Examples: `:u8`, `:bool`.
This type information lets the framework further compress the logs resulting in higher throughput.

``` rust
// arguments can be compressed into a single byte
binfmt::info!(
    "enabled: {:bool}, ready: {:bool}, timeout: {:bool}",
    enabled, ready, timeout,
);

// arguments will be type checked
binfmt::trace!("{:u16}", x);
//                       ^ must have type `u16`
```

The available types are:

- `:bool`, boolean
- `:{i,u}{8,16,32}`, standard integer types
- `:{i,u}24`, 32-bit integer truncated to 24 bits
- `:[u8; N]`, byte array
- `:[u8]`, byte slice
- `:str`, string slice

### Bitfields

`:m..n` is the bitfield formatting parameter.
When paired with a positional parameter it can be used to display the bitfields of a register.

``` rust
// -> TRACE: PCNF1 { MAXLEN: 125, STATLEN: 3, BALEN: 0b010 }
binfmt::trace!(
    "PCNF1: {{ MAXLEN: {0:0..8}, STATLEN: {0:8..16}, BALEN: {0:16..19} }}",
    //                  ^                  ^                 ^ same argument
    pcnf1, // <- type must be `u32`
);
```

### `derive(Format)`

The preferred way to implement the `Format` trait for a struct or enum is to use the `derive` attribute.

``` rust
#[derive(Format)]
struct Header {
    source: u8,
    destination: u8,
    sequence: u16,
}

#[derive(Format)]
enum Request {
    GetDescriptor { descriptor: Descriptor, length: u16 },
    SetAddress { address: u8 },
}
```

### `write!`

When implementing the `Format` trait manually the `write!` macro must be used to log the data.
This macro takes a `Formatter` as its first argument.

``` rust
/// Packet configuration register 1
pub struct PCNF1 { value: u32 }

impl binfmt::Format for PCNF1 {
    fn fmt(&self, f: &mut binfmt::Formatter) {
        binfmt::write!(
            f,
            "PCNF1: {{ MAXLEN: {0:0..8}, STATLEN: {0:8..16}, BALEN: {0:16..19} }}",
            self.value,
        );
    }
}
```

### Logging level filtering

`binfmt` supports 5 different logging levels listed below from lowest severity to highest severity:

- Trace
- Debug
- Info
- Warn
- Error

By default all logging is *disabled*.
The amount of logging to perform can be controlled at the crate level using Cargo features.

All crates, both libraries and applications, that use any of `binfmt` logging macros MUST expose these Cargo features:

``` toml
[features]
binfmt-default = [] # log at INFO, or TRACE, level and up
binfmt-trace = []   # log at TRACE level and up
binfmt-debug = []   # log at DEBUG level and up
binfmt-info = []    # log at INFO level and up
binfmt-warn = []    # log at WARN level and up
binfmt-error = []   # log at ERROR level
```

These features must only be enabled by the top level *application* crate as shown below.

``` toml
[dependencies]
usb-device = { version, "0.3.0", features = ["binfmt-default"] }
#                                            ^^^^^^^^^^^^^^^

[features]
default = ["binfmt-trace"]
#          ^^^^^^^^^^^^^^
```

When only "binfmt-default" is enabled the crate will:

- log at the TRACE level and up if `debug-assertions = true` (`dev` profile), or
- log at the INFO level and up if `debug-assertions = false` (`release` profile)

When any of the other features is enabled the crate will log at that, and higher, severity regardless of the state of `debug-assertions` or "binfmt-default".

### Global logger

*Applications* that, directly or transitively, use any of `binfmt` logging macros need to define a `#[global_logger]` or include one in their dependency graph.
This is similar to how the `alloc` crate depends on a `#[global_allocator]`.

The `global_logger` defines how data is moved from the *device*, where the application runs, to the host, where logs will be formatted and displayed.
`global_logger` is transport agnostic: you can use a serial interface, serial over USB, RTT, semihosting, Ethernet, 6LoWPAN, etc. to transfer the data.

The `global_logger` interface comprises two traits, `Write` and `Logger`, and one attribute, `#[global_logger]`.

The `Write` trait specifies how the data is put on the wire.
The `write` method is not allowed to fail.
Buffering, rather than waiting on I/O, is recommended.
If using buffering `write` should not overwrite old data as this can corrupt log frames and most printers cannot deal with incomplete log frames.

The `Logger` specifies how to acquire and release a handle to a global logger.
See the API documentation for more details about the safety requirements of the acquire-release mechanism.

Finally, `#[global_logger]` specifies which `Logger` implementation will be used by the application.
`#[global_logger]` must be used on a *unit* struct, a struct with no fields.
This struct must implement the `Logger` trait.
It's recommended that this struct is kept private.
Only a single `#[global_logger]` struct can appear in the dependency graph of an application.
The `global_logger` should be selected *at the top*, in the application crate.

There are two general ways to implement a `global_logger`.

#### Single logging channel

The first form uses a single channel.
This means that all execution contexts (i.e. threads OR `main` + interrupt handlers) use the same logging channel.
In an application that uses interrupts this means that `acquire` must disable interrupts and `release` must re-enable interrupts.
This synchronizes access to the single channel, from contexts running at different priority levels.

`binfmt-semihosting` is an example of this single logging channel approach.

#### Multiple logging channel

The other approach uses multiple logging channels: e.g. one for each priority level in an application that uses interrupts.
With this approach logging can be made lock-free: interrupts are not disabled while logging data.
This approach requires channel multiplexing in the transport layer.
RTT, for example, natively supports multiple channels so this is not an issue, but other transports, like ITM, will require that each log frame to be tagged with the channel it belongs to (e.g. one logging channel = ITM channel).
The trade-off of using more channels are:
- higher memory usage on the target, for buffering, and/or
- lower overall throughput, as either different channels need to be polled or log frames need to be tagged with the channel they belong to

### Timestamp

*Applications* that, directly or transitively, use any of `binfmt` logging macros need to define a `#[timestamp]` function or include one in their dependency graph.

All logs are timestamped.
The `#[timestamp]` function specifies how the timestamp is computed.
This function must have signature `fn() -> u64` and on each invocation *should* return a non-decreasing value.
The function is not `unsafe` meaning that it must be thread-safe and interrupt-safe.

To omit timestamp information use this `#[timestamp]` function:

``` rust
#[binfmt::timestamp]
fn timestamp() -> u64 {
    0
}
```

A simple `timestamp` function that does not depend on device specific features and it's good enough for development is shown below:

``` rust
#[binfmt::timestamp]
fn timestamp() -> u64 {
    static COUNT: AtomicU32 = AtomicU32::new(0);
    COUNT.fetch_add(1, Ordering::Relaxed)
}
```

> NOTE in long lived applications a 64-bit monotonic counter should be implemented

A `timestamp` function that uses a device-specific monotonic timer can directly read a MMIO register.
It's OK if the function returns `0` while the timer is disabled.

``` rust
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

> NOTE again this should be extended to 64-bit in long lived applications

### Linker script

*Applications* MUST pass the `-Tbinfmt.x` flag to the linker.
This should be done in the `.cargo/config` file.

``` toml
[target.thumbv7em-none-eabi] # compilation target
rustflags = [
  "-C", "link-arg=-Tbinfmt.x", # required by binfmt <- add this

  "-C", "link-arg=-Tlink.x", # required by cortex-m-rt
]
```

> NOTE(japaric) iirc, if omitted failure on Cortex-M is linker error; x86, OTOH, links but doesn't work

### Printers

*Printers* are *host* programs that receive log data, format it and display it.
The following printers are currently available:

- `qemu-run`, parses data sent over semihosting (ARM Cortex-M only)
- `probe-run`, parses data sent over RTT (ARM Cortex-M only)

## Design / implementation notes

> NOTE(japaric) this part is meant to be a reference. You do not need to grok all this right off the bat.

### Optimization goals

`binfmt` optimizes for data throughput first and then for runtime cost.

### Constraints

#### No double compilation

Say you want print logs from target/device app that uses crate `foo`.
That crate `foo` uses the `Format` trait on some of its data structures.
In this scenario we want to *avoid* having to compile `foo` for the host.
In other words, `foo` should only be (cross) compiled for the target device.

This is the biggest difference between `binfmt` and some `serde` library that does binary serialization.
The `serde` library requires a `Deserialize` step that requires compiling `foo` for the host (see `derive(Serialize)`).
`binfmt` avoids this by placing all the required information *for formatting* in a "side table" (see string interning below).
This comes with the downside that the host can only perform limited actions on the data it receives from the device: namely formatting.
The host cannot invoke `foo::Struct.method()` for example but that may not even be a sensible operation on the host anyways, e.g. `foo::USB::RegisterValue.store_volatile()`.

We want to avoid this "double" compilation (cross compile for the target *and* compile for the host) because:
- it doubles compilation time (wait time)
- compiling device-specific code for the host can become a headache quickly: see inline/external assembly, build scripts, etc.

### Interning

All string literals are interned in a custom ELF section.
This has proven to be the way that requires the less post-processing and implementation work.
It is not without downsides as we'll see.

The basic pattern for interning a string is this:

``` rust
#[export_name = "the string that will be interned"]
#[link_section = ".my_custom_section.some_unique_identifier"]
//             ^ this is the INPUT linker section
static SYM: u8 = 0;

// index of the interned string
let index = &SYM as *const u8 as usize;
```

A linker script is required to group all these strings into a single OUTPUT linker section:

``` text
SECTIONS
{
  /* NOTE: simplified */
  .my_custom_section /* <- name of the OUTPUT linker section */
    (INFO) /* <- metadata section: not placed in Flash */
    : 0 /* <- start address of this section */
  {
    *(.my_custom_section.*); /* <- name of the INPUT linker section */
  /*^                    ^ glob pattern */
  /*^ from any object file (~= crate) */
  }
}
```

With this linker script the linker will tightly pack all the interned strings in the chosen linker section.
The linker will also discard strings that end no being used in the final binary AKA "garbage collection".
Garbage collection will only work correctly if every string is placed in a *different* INPUT linker section.

After you have linked the program you can display the interned strings using the `nm` tool.

``` console
$ arm-none-eabi-nm -CSn elf-file
00000000 00000001 N USB controller is ready
00000001 00000001 N entering low power mode
00000002 00000001 N leaving low power mode
(..)
```

The `nm` shows all the *symbols* in the ELF file.
In ELF files one function = one symbol and one static variable = one symbol.
So function `foo` will show as `crate_name::module_name::foo` in the `nm` output; same thing with a static variable `X`.

The four columns in the output, from left to right, contain:
- the address of the symbol
- the size of the symbol in bytes
- the type of the symbol
- the symbol name

As you can see the interned string is the symbol name.
Although we cannot write:

``` rust
static "USB controller is ready": u8 = 0;
```

We can write:

``` rust
#[export_name = "USB controller is ready"]
static SYM: u8 = 0;
```

The next thing to note is that each interned string symbol is one byte in size (because `static SYM` has type `u8`).
Thanks to this the addresses of the symbols are consecutive: 0, 1, 2, etc.

#### Dealing with duplicates

The linker hates it when it finds two symbol that have the same name.
For example, this is an error:

``` rust
#[no_mangle]
static X: u32 = 0;

#[export_name = "X"]
static Y: u32 = 0; //~ error: symbol `X` is already defined
```

This produces two symbols with the name "X".
`rustc` catches this issue early and reports an error at *compile* time.

How can this occur in logging?
The user may write:

``` rust
fn foo() {
    binfmt::info!("foo started ..");
    // ..
    binfmt::info!(".. DONE"); // <-
}

fn bar() {
    binfmt::info!("bar started ..");
    // ..
    binfmt::info!(".. DONE"); // <-
}
```

Because macros are expanded in isolation *each* `info!(".. DONE")` statement will produce this to intern its string:

``` rust
#[export_name = ".. DONE"]
#[link_section = ".."]
static SYM: u8 = 0;
```

which results in a collision.

To avoid this issue we suffix each interned string a suffix of the form: `@1379186119` where the number is randomly generated.
Now these two macro invocations will produce something like this:

``` rust
// first info! invocation
#[export_name = ".. DONE@1379186119"]
#[link_section = ".."]
static SYM: u8 = 0;

// ..

// second info! invocation
#[export_name = ".. DONE@346188945"]
#[link_section = ".."]
static SYM: u8 = 0;
```

These symbols do not collide and the program will link correctly.

Why use the `@` character?
The `@` character is special in ELF files; it is used for *versioning* symbols.
In practice what this means is that what comes after the `@` character is *not* part of the symbol name.
So if you run `nm` on the last Rust program you'll see:

``` console
$ arm-none-eabi-nm -CSn elf-file
00000000 00000001 N .. DONE
(..)
00000002 00000001 N .. DONE
(..)
```

That is the random number (the version) won't show up there.

> NOTE(japaric) Also I didn't see a straightforward way to extract symbol versions from ELF metadata.

Because duplicates are kept in the final binary this linker-based interner is not really an interner.
A proper interner returns the same index when the same string is interned several times.

> NOTE(japaric) AFAIK it is not possible to deduplicate the symbols with this proc-macro + linker implementation

Because `@` is special it is not allowed in format strings.
So this code is considered an error:

``` console
binfmt::info!("DONE @ foo");
//                  ^ error: `@` not allowed in format strings
```

#### Logging levels

`binfmt` supports several logging levels.
To avoid serializing the logging level at runtime (that would reduce throughput), interned strings are clustered by logging level.

The `binfmt` linker script looks closer to this:

``` text
SECTIONS
{
  .binfmt (INFO) : 0
  {
    *(.binfmt.error.*); /* cluster of ERROR level log strings */

    _binfmt_warn = .; /* creates a symbol between the clusters */

    *(.binfmt.warn.*); /* cluster of WARN level log strings */

    _binfmt_info = .;
    *(.binfmt.info.*);

    _binfmt_debug = .;
    *(.binfmt.debug.*);

    _binfmt_trace = .;
    *(.binfmt.trace.*);
  }
}
```

And the string interning that each logging macro does uses a different input linker section.
So this code:

``` rust
binfmt::warn!("Hello");
binfmt::warn!("Hi");

binfmt::error!("Good");
binfmt::error!("Bye");
```

Would expand to this:

``` rust
// first warn! invocation
#[export_name = "Hello@1379186119"]
#[link_section = ".binfmt.warn.1379186119"]
static SYM: u8 = 0;

// ..

// first error! invocation
#[export_name = "Bye@346188945"]
#[link_section = ".binfmt.error.346188945"]
static SYM: u8 = 0;
```

Then after linking we'll see something like this in the output of `nm`:

``` console
$ arm-none-eabi-nm -CSn elf-file
00000000 00000001 N Bye
00000001 00000001 N Good
00000002 00000000 N _binfmt_warn
00000002 00000001 N Hi
00000003 00000001 N Hello
00000003 00000000 N _binfmt_info
(..)
```

There you can see that ERROR level logs are clustered at the beginning.
After that cluster comes the cluster of WARN level logs.
Between the two clusters you see the zero-sized `_binfmt_warn` symbol.

We know before-hand the name of the `_binfmt_*` symbols which are used as delimiters.
We can look their addresses first and when we lookup a string index in this table we can compare it to those addresses to figure the logging level it corresponds to.

- if `index < indexof(_binfmt_warm)` then ERROR log level
- if `indexof(_binfmt_warn) <= index < indexof(_binfmt_info)` then WARN log level

And so on so forth.

### Serialization

In this section we'll see how log data is "put on the wire".

#### Interned strings

Let's ignore timestamps for now and also ignore how access to the global logger is synchronized.
This is the simplest case: logging a string literal with no formatting.

``` rust
binfmt::info!("Hello, world!");
```

As we saw in the previous section this string will get interned.
Interning converts the string into a `usize` index.
This `usize` index will be further compressed using [LEB128].
Some examples: (values on the right are `u8` arrays)

[LEB128]: https://en.wikipedia.org/wiki/LEB128

- `1usize` -> `[1]`
- `127usize` -> `[127]`
- `128usize` -> `[128, 1]`
- `255usize` -> `[255, 1]`

Because string indices start at zero and it's unlikely that a program will intern more than 2^14 strings string literals will be serialized as 1 or 2 bytes indices.

#### Integers

Integers will be serialized in little endian order using `to_le_bytes()`.
`usize` and `isize` values will be subject to LEB128 compression.

``` rust
binfmt::error!("The answer is {:i16}!", 300);
// on the wire: [3, 44, 1]
//  string index ^  ^^^^^ `300.to_le_bytes()`
//  ^ = intern("The answer is {:i16}!")

binfmt::error!("The answer is {:u24}!", 131000);
// on the wire: [4, 184, 255, 1]
//                  ^^^^^^^^^^^ 131000.to_le_bytes()[..3]

binfmt::error!("The answer is {:usize}!", 131000);
// on the wire: [4, 184, 255, 1]
//                  ^^^^^^^^^^^ 131000.to_le_bytes()[..3]
```

> NOTE(japaric) unclear to me if LEB128 encoding (more compression but more) `u16` and `u32` is worth the trade-off

> TODO(japaric) evaluate [zigzag encoding][zigzag] for `isize`?

[zigzag]: https://developers.google.com/protocol-buffers/docs/encoding

#### Slices

For slices (`{:[u8]}`) the length is LEB128 encoded and serialized first and then followed by the slice data.

``` rust
binfmt::error!("Data: {:[u8]}!", [0, 1, 2]);
// on the wire: [1, 3, 0, 1, 2]
//  string index ^  ^  ^^^^^^^ the slice data
//   LEB128(length) ^
```

#### String values
Strings that are passed directly (i.e. not as indices of interned strings) as format string parameters (`{:str}`) must be prefixed with their LEB128 encoded length. This behavior is analogous to that of Slices.

``` rust
binfmt::error!("Hello, {:str}!", [b'w', b'o', b'r', b'l', b'd']);
// on the wire: [1, 5, 199, 111, 114, 108, 100]
//  string index ^  ^  ^^^^^^^^^^^^^^^^^^^^^^^ the slice data (byte literals are ascii chars)
//   LEB128(length) ^
```

#### Arrays

For arrays (`{:[u8; N]}`) the length is not serialized.

``` rust
binfmt::error!("Data: {:[u8; 3]}!", [0, 1, 2]);
// on the wire: [1, 0, 1, 2]
//  string index ^  ^^^^^^^ the slice data
```

#### Bitfields

The integer argument is serialized in little endian format (`to_le_bytes`).

``` rust
binfmt::error!("l: {0:0..8}, m: {0:8..12}, h: {:12..16}", x /*: u16*/);
// on the wire: [1, 1, 2]
//  string index ^  ^^^^ `u16::to_le_bytes(x)`
```

#### Bool compression

Booleans are grouped in bytes, bitflags-style.

``` rust
binfmt::error!("x: {:bool}, y: {:bool}, z: {:bool}", false, false, true);
// on the wire: [1, 0b100]
//  string index ^  ^^^^^ the booleans: `0bzyx`
```

When mixed with other data, the first `{:bool}` allocates an output byte that
fits up to 7 more bools.

``` rust
binfmt::error!("x: {:bool}, y: {:u8}, z: {:bool}", false, 0xff, true);
// on the wire: [1, 0b10, 0xff]
//  string index ^  ^^^^^ ^^^^ u8
//                  |
//                  the booleans: `0bzx`
```

#### `Format`

The untyped argument (`:?`) requires one level of indirection during serialization.

First let's see how a primitive implements the `Format` trait:

``` rust
impl Format for u8 {
    fn format(&self, f: &mut Formatter) {
        binfmt::write!(f, "{:u8}", self)
        // on the wire: [1, 42]
        //  string index ^  ^^ `self`
        //  ^ = intern("{:u8}")
    }
}
```

`Format` will use the `write!` macro.
This will send the string index of `{:u8}` followed by the one-byte data.
In general, `write!` can use `{:?}` so `Format` nesting is possible.

Now let's look into a log invocation:

``` rust
binfmt::error!("The answer is {:?}!", 42u8);
// on the wire: [2, 1, 42]
//  string index ^  ^^^^^ `42u8.format(/*..*/)`
//  ^ = intern("The answer is {:?}!")
```

This will send the string index of "The answer is {:?}!" and invoke the argument's `Format::format` method.

> NOTE(japaric) might be best *not* to implement `Format` for primitive integers.
> `{:u8}` and similar use less bandwidth and should be preferred.
> The `derive(Format)` uses typed parameters instead of `{:?}` where possible

> TODO(japaric) a naive `[T]`'s `Format` implementation (`slice.for_each(format)`) has high overhead: the string index of e.g. `{:u8}` would be repeated N times.
> We'll need to some specialization to avoid that repetition.

### Single `Format` trait

`core::fmt` has several formatting traits, like `Hex` and `Bin`.
These appear as different formatting parameters, like `:x` and `:b`, in format strings and change how integers are formatted: `15` vs `0xF` vs `0b1111`.

`binfmt` does not have all these formatting traits.
The rationale is that the device should not make the decision about how an integer is formatted.
The formatting is done in the host so the host should pick the format.
With interactive displays, e.g. web UI, it even becomes possible to change the format on demand, e.g. click the number to change from decimal to hexadecimal representation.

### Timestamps

In the current implementation timestamps are absolute (time elapsed since the start of the program) and in microseconds.
Timestamps are LEB128 encoded before serialization.

> TODO we may want to consider using delta encoding in the future

### Global logger

The global logger needs to operate correctly (be memory safe and not interleave log data) in presence of race conditions and re-entrant invocations.
Race conditions can be avoided with mutexes but re-entrancy can occur even if mutexes are used and shouldn't result in deadlocks.

#### Re-entrancy

Where can re-entrancy occur?
Turns out that with global singletons it can occur about anywhere; you don't need interrupts (preemption) to cause re-entrancy.
See below:

``` rust
binfmt::info!("The answer is {:?}!", x /*: Struct */);
```

As you have seen before this will first send the string index of "The answer is {:?}!" and then call `x`'s `Format::format` method.
The re-entrancy issue arises if the `Format` implementation calls a logging macro:

``` rust
impl Format for X {
    fn format(&self, f: &mut Formatter) {
        //           ^ this is a handle to the global logger
        binfmt::info!("Hello!");
        // ..
    }
}
```

`f` is a handle to the global logger.
The `info!` call inside the `format` method is trying to access the global logger again.
If `info!` succeeds then you have two exclusive handles (`&mut Formatter`) to the logger and that's UB.
If `info!` uses a spinlock to access the logger then this will deadlock.

#### Acquire-release

One solution to the re-entrancy issue that's deadlock-free is to make the log macros *take* the logger and hold it until it's done with it.
In case of nesting any inner take attempt will silently fail.

So the log macros may expand to something like this:
(let's ignore data races / race conditions for now)

``` rust
if let Some(logger) = Logger::acquire() {
    logger.serialize_interned_string_and_etc();
    release(logger); // <- logger can be acquired again after this
} else {
    // silent failure: do nothing here
}
```

This means that invoking logging macros from `Format` implementations will silently fail.
But note that allowing such operation would result in interleaving of log frames.
To a decoder/parser interleaved log frames are the same as corrupted log frames.
So we actually want to forbid this operation.

#### Evaluation order

Consider this log invocation:

``` rust
binfmt::info!("x={:?}", foo());

fn foo() {
    binfmt::info!("Hello");
}
```

Depending on *when* `foo` is invoked this can result in potential re-entrancy / nesting and cause `info!("Hello")` to be lost.
So we'll make the macro evaluate format arguments *before* the acquire operation.
Something like this:
(`core::fmt` does a similar `match` operation)

``` rust
match (foo()) { // evaluate formatting arguments
    (_0) => {
        if let Some(logger) = Logger::acquire() {
            // serialize `_0`, etc.
        }
    }
}
```

#### Preemption

Preemption can also result in re-entrancy.
How to deal with it?
Assuming single-core systems there are two approaches:

1. Disable interrupts in `acquire`; re-enable them in `release`. This means that the logging macros block higher priority interrupts.

2. Have a separate logger per priority level. `acquire` and `release` are now lock-free and don't block interrupts. This requires multiplexing in the transport.

### Deserialization

The host has received the log data (binary data).
How to make sense of it?

Let's assume:
- no data loss during transport (reliable transport)
- no interleaving of log frames (no nesting of logging macros)

> NOTE(japaric) adding error detection to the format is a challenge for some other day

With these assumptions the decoder can expect the stream of log data to be a series of *log frames*.

#### Log frames

Each log statement produces one log frame.
Consider this log call:
(let's include the timestamp this time)

``` rust
binfmt::info!("answer={:u8}", 42);
// on the wire: [2, 125, 42] <- arguments
//  string index ^  ^^^ timestamp
```

A log frame will consist of:

- A string index that must be either of the error, warn, info, debug or trace kind.
  - String indices generated by `write!` (used in `Format` implementations) are of a different kind
- A timestamp (LEB128 encoded)
- Zero or more formatting arguments

To be able to decode the last component the host will have to lookup the format string, whose index is the first part of the log frame, and parse it.
Parsing that string will tell the host how many and how big (in bytes) the formatting arguments are.

#### Lookup

We have so far looked at the string table using `nm`.
Programmatically the table can be found in the `.symtab` section.
Each [entry] in this table represents a symbol and each entry has:
- `shndx`, a section header index (?). This should match the index of the `.binfmt` section.
- `value`, this is the address of the symbol. For `.binfmt`, this is the string index
- `name`, an index into some data structure full of strings. `get_name` returns the interned string
- the other info is not relevant

[entry]: https://docs.rs/xmas-elf/0.7.0/xmas_elf/symbol_table/trait.Entry.html
