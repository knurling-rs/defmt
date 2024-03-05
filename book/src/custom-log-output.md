# Customizing the log output

The way a printer outputs logs can be customized by providing a format string to `defmt`. The format string takes a set of metadata and format specifiers which can be used to include or exclude certain information when printing the logs.

## Basics

> The following log will be used as reference in the examples below: `defmt::debug!("hello");`

The simplest format string is `"{s}"`. This prints the log and nothing else:

```text
hello
```

Arbitrary text can be added to the format string, which will be printed as specified with each log.
For example, `"Log: {s}"`:

```text
Log: hello
```

Multiple specifiers can be included within a format string, in any order. For example `"[{L}] {s}"` prints:

```text
[DEBUG] hello
```

## Metadata specifiers

There are several metadata specifiers available that can be used in a format string.

#### Log - `{s}`

This specifier prints the actual log contents. For `defmt::info!("hello");`, this specifier prints `hello`.

#### Crate name - `{c}`

This specifier prints the name of the crate where the log is coming from.

#### File name - `{f}`

For a log coming from a file `/path/to/crate/src/foo/bar.rs`, this specifier prints `bar.rs`.

This specifier can be used to print more detailed parts of the file path. The number of `f`s in the specifier determines how many levels up the path should be printed. For example, `{ff}` prints `foo/bar.rs`, and `{fff}` prints `src/foo/bar.rs`.

#### File path - `{F}`

For a log coming from a file `/path/to/crate/src/foo/bar.rs`, this specifier prints `/path/to/crate/src/foo/bar.rs`.

#### Line number - `{l}`

This specifier prints the line number where the log is coming from.

#### Log level - `{L}`

This specifier prints the log level. The log level is padded to 5 characters by default, for alignment purposes. For `defmt::info!("hello);`, this prints `INFO `.

#### Module path - `{m}`

This specifier prints the module path of the function where this log is coming from. This prints `my_crate::foo::bar` for the log shown below:

```ignore
// crate: my_crate
mod foo {
    fn bar() {
        defmt::info!("hello");
    }
}
```

#### Timestamp - `{t}`

This specifier prints the timestamp at which a log was logged, as formatted by `defmt::timestamp!`.

## Customizing log segments

The way a metadata specifier is printed can be customized by providing additional, optional format specifiers.

Format parameters are provided by adding the formatting parameters after the metadata specifier, separated by colons (`:`), like this: `"{L:bold:5} {f:white:<10} {s}"`.

### Color

A log segment can be specified to be colored by providing a color in the format parameters.

There are three different options for coloring a log segment:
- using a supported color such as `red` or `green`.
- `severity` colors the log segment using the predefined color for the log level of the log.
- `werror` is similar to `severity`, but it only applies the color if the log level is `WARN` or `ERROR`.

Only one coloring option can be provided in format parameters for a given log segment, i.e. `{L:red:green}` is not supported.

The following colors are supported in the format parameters:
- `black`
- `red`
- `green`
- `yellow`
- `blue`
- `magenta`
- `purple`
- `cyan`
- `white`
- `bright black`
- `bright red`
- `bright green`
- `bright yellow`
- `bright blue`
- `bright magenta`
- `bright purple`
- `bright cyan`
- `bright white`

### Styles

A log segment can be specified to be printed with a given style by providing a style specifier in the format parameters.

The style specifier must be one of the following strings:
- `bold`
- `italic`
- `underline`
- `strike`
- `dimmed`

Multiple styles can be applied to a single log segment, but they must not be repeated, i.e.
`"{s:bold:underline:italic}"` is allowed, but `"{s:bold:bold}"` isn't.

### Width, alignment and padding

A log segment can be specified to be printed with a given minimum width and alignment by providing a format parameter.

The alignment can be specified to be left (`<`), right (`>`), or center-aligned (`^`). If no alignment is specified, left alignment is used by default.

The minimum width is specified after the alignment. For example, `"{L} {f:>10}: {s}"` is printed as follows:

```text
[DEBUG]    main.rs: hello
```

The log segment is padded with spaces by default in order to fill the specified segment width. A specifier can be padded with zeros by prefixing the width specifier with a zero, e.g. `{l:03}` prints a line number 24 as `024`.

## Nested formatting

Log segments can be grouped and formatted together by nesting formats. Format parameters for the grouped log segments must be provided after the group, separated by `%`.

Nested formats allow for more intricate formatting. For example, `"{[{L:bold}]%underline} {s}"` prints

```text
[DEBUG] hello
```

where only `DEBUG` is formatted bold, and `[DEBUG]` is underlined.

Formats can be nested several levels. This provides a great level of flexibility to customize the logger formatting.
For example, the width and alignment of a group of log segments can be specified with nested formats.
`"{{[{L}]%bold} {f}:{l}%35} {s}"` prints something like this:

```text
[DEBUG] main.rs:20                  hello
[DEBUG] goodbye.rs:304              goodbye
```

## Restrictions

- Format strings *must* include the `{s}` metadata specifier.
- At the moment it is not possible to escape curly brackets (i.e. `{`, `}`) in the format string, therefore curly brackets cannot be printed as part of the logger format.
- The same restriction exists for the `%` character.

## Passing log format to printers

The format string can be passed to `probe-rs`, `probe-run` and `defmt-print` using the `--log-format` option.

This option can be passed to the printer in `.cargo/config.toml`, but due to limitations in `cargo`, the command has to be split as follows:

```toml
# .cargo/config.toml

runner = [
  "probe-rs",
  "run",
  "--chip",
  "nRF52840_xxAA",
  "--log-format",
  "{L} {s}",
]
```

The format of the logs printed by the host can also be customized using the `--host-log-format` option.
