# Interning

> ⚠️ The design and implementation chapter is outdated ⚠️

All string literals are interned in a custom ELF section.
This has proven to be the way that requires the less post-processing and implementation work.
It is not without downsides as we'll see.

The basic pattern for interning a string is this (although note that what `defmt` actually does is more complicated - see the [Dealing with duplicates](./duplicates.md) section for more details):

``` rust,no_run,noplayground
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
    .my_custom_section 0 (INFO) :
    /*                    ^^^^^^ metadata section: not placed in Flash  */
    /*                 ^ start address of this section                  */
    /* ^^^^^^^^^^^^^^^ name of the OUTPUT linker section                */
    {
        *(.my_custom_section);  
        *(.my_custom_section.*);
    /*                       ^ glob pattern for sub-sections  */
    /*     ^^^^^^^^^^^^^^^^^ name of the INPUT linker section */
    /*  ^ from any object file (~= crate)                     */
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

``` rust,ignore
static "USB controller is ready": u8 = 0;
```

We can write:

``` rust
#[export_name = "USB controller is ready"]
static SYM: u8 = 0;
```

The next thing to note is that each interned string symbol is one byte in size (because `static SYM` has type `u8`).
Thanks to this the addresses of the symbols are consecutive: 0, 1, 2, etc.
