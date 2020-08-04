# Logging levels

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
# extern crate binfmt;
binfmt::warn!("Hello");
binfmt::warn!("Hi");

binfmt::error!("Good");
binfmt::error!("Bye");
```

Would expand to this:

``` rust
// first warn! invocation
{
    #[export_name = "Hello@1379186119"]
    #[link_section = ".binfmt.warn.1379186119"]
    static SYM: u8 = 0;
}

// ..

// first error! invocation
{
    #[export_name = "Bye@346188945"]
    #[link_section = ".binfmt.error.346188945"]
    static SYM: u8 = 0;
}
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
