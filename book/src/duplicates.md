# Dealing with duplicates

The linker hates it when it finds two symbol that have the same name.
For example, this is an error:

``` rust,compile_fail
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
# extern crate binfmt;
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
{
    #[export_name = ".. DONE@1379186119"]
    #[link_section = ".."]
    static SYM: u8 = 0;
}

// ..

// second info! invocation
{
    #[export_name = ".. DONE@346188945"]
    #[link_section = ".."]
    static SYM: u8 = 0;
}
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
