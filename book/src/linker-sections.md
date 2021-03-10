# Logging levels

`defmt` supports several logging levels.
To avoid serializing the logging level at runtime (that would reduce throughput), interned strings are clustered by logging level.

The `defmt` linker script looks closer to this:

``` text
SECTIONS
{
  .defmt (INFO) : 0
  {
    *(.defmt.error.*); /* cluster of ERROR level log strings */

    _defmt_warn = .; /* creates a symbol between the clusters */

    *(.defmt.warn.*); /* cluster of WARN level log strings */

    _defmt_info = .;
    *(.defmt.info.*);

    _defmt_debug = .;
    *(.defmt.debug.*);

    _defmt_trace = .;
    *(.defmt.trace.*);
  }
}
```

And the string interning that each logging macro does uses a different input linker section.
So this code:

``` rust
# extern crate defmt;
defmt::warn!("Hello");
defmt::warn!("Hi");

defmt::error!("Good");
defmt::error!("Bye");
```

Would expand to this:

``` rust
// first warn! invocation
{
    #[export_name = "{\"package\":\"my-app\",\"tag\":\"defmt_warn\",\"data\":\"Hello\",\"disambiguator\":\"8864866341617976971\"}"]
    #[link_section = ".defmt.{\"package\":\"my-app\",\"tag\":\"defmt_warn\",\"data\":\"Hello\",\"disambiguator\":\"8864866341617976971\"}"]
    static SYM: u8 = 0;
}

// ..

// first error! invocation
{
    #[export_name = "{\"package\":\"my-app\",\"tag\":\"defmt_error\",\"data\":\"Bye\",\"disambiguator\":\"2879057613697528561\"}"]
    #[link_section = ".defmt.{\"package\":\"my-app\",\"tag\":\"defmt_error\",\"data\":\"Bye\",\"disambiguator\":\"2879057613697528561\"}"]
    static SYM: u8 = 0;
}
```

Then after linking we'll see something like this in the output of `nm`:

``` console
$ arm-none-eabi-nm -CSn elf-file
00000000 00000001 N Bye
00000001 00000001 N Good
00000002 00000000 N _defmt_warn
00000002 00000001 N Hi
00000003 00000001 N Hello
00000003 00000000 N _defmt_info
(..)
```

There you can see that ERROR level logs are clustered at the beginning.
After that cluster comes the cluster of WARN level logs.
Between the two clusters you see the zero-sized `_defmt_warn` symbol.

We know before-hand the name of the `_defmt_*` symbols which are used as delimiters.
We can look their addresses first and when we lookup a string index in this table we can compare it to those addresses to figure the logging level it corresponds to.

- if `index < indexof(_defmt_warm)` then ERROR log level
- if `indexof(_defmt_warn) <= index < indexof(_defmt_info)` then WARN log level

And so on so forth.
