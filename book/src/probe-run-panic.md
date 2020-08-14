# Non-zero exit code

The `panic-probe` crate is a panic handler that makes the `probe-run` program exit with non-zero exit code.
This panic handler can be used to write unit tests: `assert!`-like macros will make `panic-probe` / `cargo-run` exit with non-zero exit code if they fail.

``` rust,ignore
#[entry]
fn main() -> ! {
    defmt::info!("main");
    assert!(false);
    loop { asm::bkpt() }
}
```

``` console
$ cargo run --bin panic
0.000000 INFO main
stack backtrace:
   0: 0x00000316 - __bkpt
   1: 0x00000314 - panic_probe::__cortex_m_rt_HardFault
   2: 0x000003c2 - HardFault
      <exception entry>
   3: 0x00000328 - __udf
   4: 0x0000030a - cortex_m::asm::udf
   5: 0x00000300 - rust_begin_unwind
   6: 0x000002b2 - core::panicking::panic_fmt
   7: 0x000002a8 - core::panicking::panic
   8: 0x000001a4 - panic::__cortex_m_rt_main
   9: 0x00000108 - main
  10: 0x000002de - Reset

$ echo $?
134
```
