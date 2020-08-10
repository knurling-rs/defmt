# Stack backtraces

When the firmware reaches a BKPT instruction the device halts.
The `probe-run` tool treats this halted state as the "end" of the application and exits with exit-code = 0.
Before exiting `probe-run` prints the stack backtrace of the halted program.
This backtrace follows the format of the `std` backtraces you get from `std::panic!` but includes `<exception entry>` lines to indicate where an exception/interrupt occurred.

``` rust,ignore
#[entry]
fn main() -> ! {
    binfmt::info!("main");
    SCB::set_pendsv();
    binfmt::info!("after PendSV");

    loop { asm::bkpt() }
}

#[exception]
fn PendSV() {
    binfmt::info!("PendSV");
    asm::bkpt()
}
```

``` console
$ cargo run --bin exception
0.000000 INFO main
0.000001 INFO PendSV
stack backtrace:
   0: 0x0000048a - __bkpt
      <exception entry>
   1: 0x000003d4 - _binfmt_acquire
   2: 0x0000016e - exception::__cortex_m_rt_main
   3: 0x00000108 - main
   4: 0x00000466 - Reset
```
