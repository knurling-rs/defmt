# `panic!` and `assert!`

The `defmt` crate provides its own version of `panic!`-like and `assert!`-like macros.
The `defmt` version of these macros will log the panic message using `defmt` and then call `core::panic!` (by default).
Because the panic message is formatted using `defmt!` the format string must use the same syntax as the logging macros (e.g. `info!`).

## `#[defmt::panic_handler]`

Because `defmt::panic!` invokes `core::panic!` this can result in the panic message being printed twice if your `#[core::panic_handler]` also prints the panic message.
This is the case if you use [`panic-probe`] with the `print-defmt` feature enabled but not an issue if you are using the [`panic-abort`] crate, for example.

[`panic-probe`]: https://crates.io/crates/panic-probe
[`panic-abort`]: https://crates.io/crates/panic-abort

To avoid this issue you can use the `#[defmt::panic_handler]` to *override* the panicking behavior of `defmt::panic`-like and `defmt::assert`-like macros.
This attribute must be placed on a function with signature `fn() -> !`.
In this function you'll want to replicate the panicking behavior of the Rust `#[panic_handler]` but leave out the part that prints the panic message.
For example:

<!-- NOTE(ignore) we can't compile this test because the `panic_handler` defined here collides with the one in `std` -->

``` rust, ignore
#[panic_handler] // built-in ("core") attribute
fn core_panic(info: &core::panic::PanicInfo) -> ! {
    print(info); // e.g. using RTT
    reset()
}

#[defmt::panic_handler] // defmt's attribute
fn defmt_panic() -> ! {
    // leave out the printing part here
    reset()
}
```

If you are using the `panic-probe` crate then you should "abort" (call `cortex_m::asm::udf`) from `#[defmt::panic_handler]` to match its behavior.

NOTE: even if you don't run into the "double panic message printed" issue you may still want to use `#[defmt::panic_handler]` because this way `defmt::panic` and `defmt::assert` will *not* go through the `core::panic` machinery and that *may* reduce code size (we recommend you measure the effect of the change).
