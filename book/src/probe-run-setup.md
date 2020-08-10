# Setup

The recommend way to use `probe-run` is to set as the Cargo runner of your application.
Add this line to your Cargo configuration (`.cargo/config`) file:

``` toml
[target.'cfg(all(target_arch = "arm", target_os = "none"))']
runner = "probe-run --chip $CHIP"
```

Instead of `$CHIP` you'll need to write the name of your microcontroller.
For example, one would use `nRF52840_xxAA` for the nRF52840 microcontroller.
To list all supported chips run `probe-run --list-chips`.

You are all set.
You can now run your firmware using `cargo run`.
For example,

``` rust,ignore
// a binfmt application
fn main() -> ! {
    binfmt::info!("Hello, world!");
    loop { asm::bkpt() }
}
```

``` console
$ cargo run --bin hello
     Running `probe-run target/thumbv7em-none-eabi/debug/hello`
flashing program ..
DONE
resetting device
0.000000 INFO Hello, world!
stack backtrace:
   0: 0x0000031e - __bkpt
   1: 0x000001d2 - hello::__cortex_m_rt_main
   2: 0x00000108 - main
   3: 0x000002fa - Reset
```
