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
# extern crate defmt;
fn foo() {
    defmt::info!("foo started ..");
    // ..
    defmt::info!(".. DONE"); // <-
}

fn bar() {
    defmt::info!("bar started ..");
    // ..
    defmt::info!(".. DONE"); // <-
}
```

Because macros are expanded in isolation *each* `info!(".. DONE")` statement will produce this to intern its string:

``` rust,no_run,noplayground
#[export_name = ".. DONE"]
#[link_section = ".."]
static SYM: u8 = 0;
```

which results in a collision.

To avoid this issue we store each interned string as a JSON object with 3 fields: the message itself, the name of the crate that invoked the macro, and a 64-bit integer "disambiguator".
The disambiguator is a hash of the source code location of the log statement so it should be unique per crate.
Now these two macro invocations will produce something like this:

``` rust,no_run,noplayground
// first info! invocation
{
    #[export_name = "{ \"package\": \"my-app\", \"data\": \".. DONE\", \"disambiguator\": \"1379186119\" }"]
    #[link_section = ".."]
    static SYM: u8 = 0;
}

// ..

// second info! invocation
{
    #[export_name = "{ \"package\": \"my-app\", \"data\": \".. DONE\", \"disambiguator\": \"346188945\" }"]
    #[link_section = ".."]
    static SYM: u8 = 0;
}
```

These symbols do not collide because their disambiguator fields are different so the program will link correctly.

Because duplicate strings are kept in the final binary this linker-based interner is not really an interner.
A proper interner returns the same index when the same string is interned several times.

*However*, two log statements that log the same string will often have *different* source code locations.
Assigning a different interner index to each log statement means we can distinguish between the two thus we can report their correct source code location.
