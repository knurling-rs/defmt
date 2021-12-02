#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::debug;

use defmt::write;
use defmt_semihosting as _; // global logger

#[entry]
fn main() -> ! {
    {
        #[derive(defmt::Format)]
        struct Foo {
            bar: [u8; 4],
        }

        // outputs: "Arr: [a, b, c, d]"
        defmt::info!("Arr: {:x}", [10, 11, 12, 13]);
        // outputs: "Foo: Foo { bar: [a, b, c, d] }"
        defmt::info!("Foo: {:x}", Foo { bar: [10, 11, 12, 13] });
    }

    {
        struct S1 {
            x: &'static str,
            y: u8,
        }
        impl defmt::Format for S1 {
            fn format(&self, f: defmt::Formatter) {
                write!(f, "S1 {{ x: {=str:?}, y: {=u8:?} }}", self.x, self.y)
            }
        }

        // outputs: "S { x: "hi", y: 0x2a }"
        defmt::info!("{:#x}", S1 { x: "hi", y: 42 });
    }

    {
        struct S2 {
            x: u8,
        }
        impl defmt::Format for S2 {
            fn format(&self, f: defmt::Formatter) {
                write!(f, "S2 {{ x: {=u8:#x} }}", self.x)
            }
        }

        // ignores outer :b hint, should output: "S { x: 0x2a }"
        defmt::info!("{:#b}", S2 { x: 42 });
    }

    {
        #[derive(defmt::Format)]
        struct S {
            x: &'static str,
            y: u32,
        }

        // 0.1.x version
        defmt::warn!("Debug hint: {:?}", S { x: "hello", y: 512 });
        // 0.2.x version, results in same output
        defmt::warn!(
            "   no hint: {}",
            S {
                x: "hello",
                y: 1024
            }
        );
    }

    {
        // nested struct
        struct S1 {
            x: u16,
            y: u32,
        }
        impl defmt::Format for S1 {
            fn format(&self, f: defmt::Formatter) {
                write!(f, "S1 {{ x: {=u16:#b}, y: {} }}", self.x, self.y);
            }
        }

        struct S2 {
            s: S1,
            z: u8,
        }
        impl defmt::Format for S2 {
            fn format(&self, f: defmt::Formatter) {
                write!(f, "S2 {{ s: {:?}, z: {} }}", self.s, self.z);
            }
        }

        defmt::info!(
            "{}",
            S2 {
                s: S1 { x: 4, y: 12 },
                z: 20
            }
        );
        defmt::info!(
            "{:?}",
            S2 {
                s: S1 { x: 4, y: 12 },
                z: 20
            }
        );
        defmt::info!(
            "{:#x}",
            S2 {
                s: S1 { x: 4, y: 12 },
                z: 20
            }
        );
        defmt::info!(
            "{:#b}",
            S2 {
                s: S1 { x: 4, y: 12 },
                z: 20
            }
        );
    }

    {
        #[derive(defmt::Format)]
        struct S1<T> {
            x: T,
            y: u8,
        }

        // outputs: "S { x: "hi", y: 42 }"
        defmt::info!("{}", S1 { x: "hi", y: 42 });
        // outputs: "S { x: "hi", y: 0x2a }"
        defmt::info!("{:#x}", S1 { x: "hi", y: 42 });
    }

    loop {
        debug::exit(debug::EXIT_SUCCESS)
    }
}

// like `panic-semihosting` but doesn't print to stdout (that would corrupt the defmt stream)
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        debug::exit(debug::EXIT_FAILURE)
    }
}
