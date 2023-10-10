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

- Log - `{s}``

This specifier prints the actual log contents. For `defmt::info!("hello");`, this specifier prints `hello`.

- File name - `{ƒ}`

For a log coming from a file `src/foo/bar.rs`, this specifier prints `bar.rs`.

- File path - `{F}`

For a log coming from a file `src/foo/bar.rs`, this specifier prints `src/foo/bar.rs`

- Line number - `{l}`

This specifier prints the line number where the log is coming from.

- Log level - `{L}`

This specifier prints the log level. The log level is padded to 5 characters by default, for alignment purposes. For `defmt::info!("hello);`, this prints `INFO `.

- Module path - `{m}`

This specifier prints the module path of the function where this log is coming from. This prints `my_crate::foo::bar` for the log shown below:

```
// crate: my_crate
mod foo {
    fn bar() {
        defmt::info!("hello");
    }
}
```

- Timestamp - `{t}`

This specifier prints the timestamp at which a log was logged. For a log logged with a timestamp of 123456 ms, this prints `123456`.

- Unix-style timestamp - `{T}`

This specifier prints the timestamp at which a log was logged, in Unix-style. For a log logged with a timestamp of 123456 ms, this prints `00:02:03.456`.

## Customizing log segments

 The way a metadata specifier is printed can be customized by providing additional, optional format specifiers.

 Format parameters are provided by adding the formatting parameters after the metadata specifier, separated by colons (`:`),
 like this: `"{L:bold:5} {f:white:<10} {s}"`.

 ### Color

 A log segment can be specified to be colored by providing a color in the format parameters.

 There are three different options for coloring a log segment:
 - using a supported color such `red` or `green`.
 - `severity` colors the log segment using the predefined color for the log level of the log.
 - `werror` is similar to `severity`, but it only applies the color if the log level is `WARN` or `ERROR`.

 Only one coloring option can be provided in format parameters for a given format specifier.

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

 A log segment can be specified to be printed with a given style by providing a style in the format parameters.

 The style specifier must be one of the following strings:
 - `bold`
 - `italic`
 - `underline`
 - `strike`
 - `dimmed`

 Multiple styles can be applied to a single format specifier, but they must not be repeated, i.e.
 `"{s:bold:underline:italic}"` is allowed, but `"{s:bold:bold}"` isn't.

 ### Width, alignment and padding

 A log segment can be specified to be printed with a given minimum width and alignment by providing a format parameter.

 The alignment can be specified to be left (`<`), right (`>`), or center-aligned (`^`).
 If no alignment is specified, left alignment is used by default.

 The minimum width is specified after the alignment.
 For example, `"{L} {f:>10}: {s}"` is printed as follows:

 ```text
 [DEBUG]    main.rs: hello
 ```

 The log segment is padded with spaces by default in order to fill the specified segment width. The timestamp specifier can additionally be padded with zeros by prefixing the width specifier with a zero, e.g. `{t:>06}` prints a timestamp of 1234 as `00001234`.

 If no format parameters are provided, some metadata specifiers are printed with a default width, alignment and padding for convenience.
 - `{L}` is printed as `{L:<5}`.
 - `{t}` is printed as `{t:<08}`.
 - `{T}` is printed as `{T:<12}`.

 ## Nested formatting

 Log segments can be grouped and formatted together by nesting formats. Format parameters for the grouped log segments
 must be provided after the group, separated by `%`.

 Nested formats allow for more intricate formatting. For example, `"{[{L:bold}]%underline} {s}"` prints

 ```text
 [DEBUG] hello
 ```

 where only `DEBUG` is formatted bold, and `[DEBUG]` is underlined.

 Formats can be nested several levels. This provides a great level of flexibility to customize the logger formatting.
 For example, the width and alignment of a group of log segments can be specified with nested formats.
 `"{{[{L}]%bold} {f:>20}:%<35} {s}"` prints:

 ```text
 [DEBUG]              main.rs:       hello
 ```

 ## Restrictions

 - Format strings *must* include the `{s}` metadata specifier.
 - At the moment it is not possible to escape curly brackets (i.e. `{`, `}`) in the format string, therefore curly brackets cannot be printed as part of the logger format.
 - The same restriction exists for the `%` character.
