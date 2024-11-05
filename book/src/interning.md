# Interning

All string literals are interned in a custom ELF section.
This has proven to be the way that requires the less post-processing and implementation work.
It is not without downsides as we'll see.

The basic pattern for interning a string is this:

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

## Encoding

Storing strings as-is in symbol names can cause compatibility problems, since it can contain any arbitrary character. For example:

- The `'@'` character can't be used in symbol names because it's reserved for denoting symbol versions.
- The double-quote character `'"'` causes issues with escaping if you use it with `sym` inside an `asm!()` call.

To deal with this, as of Wire Format Version 5, strings are encoded to bytes as UTF-8, and then the bytes are hex-encoded.
The symbol is prefixed with `__defmt_hex_` to denote it's is hex-encoded, and to allow for future expansion.


``` rust
#[export_name = "__defmt_hex_55534220636f6e74726f6c6c6572206973207265616479"]
static SYM: u8 = 0;
```
