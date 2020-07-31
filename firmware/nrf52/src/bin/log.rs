#![no_std]
#![no_main]

use binfmt::Format;
use binfmt_rtt as _; // <- global logger
use cortex_m::asm;
use cortex_m_rt::entry;
use embedded_hal::timer::CountDown as _;
use nrf52840_hal::{
    target::{self, TIMER0},
    timer::Timer,
};
use panic_halt as _; // <- panicking behavior

#[no_mangle]
fn _binfmt_timestamp() -> u64 {
    unsafe {
        let timer = core::mem::transmute::<_, TIMER0>(());
        timer.tasks_capture[1].write(|w| w.bits(1));
        timer.cc[1].read().bits() as u64
    }
}

#[entry]
fn main() -> ! {
    // start monotonic counter
    let periph = target::Peripherals::take().unwrap();

    let mut timer = Timer::periodic(periph.TIMER0);
    timer.start(u32::max_value());
    drop(timer); // will only be accessed from `_binfmt_timestamp`

    binfmt::info!("Hello!");
    binfmt::info!("World!");
    binfmt::info!("The answer is {:u8}", 42);

    #[derive(Format)]
    struct S {
        x: u8,
        y: u16,
    }

    #[derive(Format)]
    struct X {
        y: Y,
    }

    #[derive(Format)]
    struct Y {
        z: u8,
    }

    binfmt::info!("{:?}", S { x: 1, y: 256 });
    binfmt::info!("{:?}", X { y: Y { z: 42 } });

    loop {
        asm::bkpt()
    }
}
