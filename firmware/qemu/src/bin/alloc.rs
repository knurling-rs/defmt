#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use core::sync::atomic::{AtomicU32, Ordering};

use cortex_m_rt::entry;
use cortex_m_semihosting::debug;
use defmt::Format;

use defmt_semihosting as _; // global logger

#[entry]
fn main() -> ! {
    if_alloc::init();

    defmt::info!("Box<u32>: {:?}", Box::new(42u32));
    defmt::info!("Box<Box<u32>>: {:?}", Box::new(Box::new(1337u32)));
    defmt::info!("Box<NestedStruct>: {:?}", Box::new(nested()));
    defmt::info!("Rc<u32>: {:?}", Rc::new(42u32));
    defmt::info!("Arc<u32>: {:?}", Arc::new(42u32));
    defmt::info!("Vec<u32>: {:?}", vec![1u32, 2, 3, 4]);
    defmt::info!("Vec<i32>: {:?}", vec![-1i32, 2, 3, 4]);
    defmt::info!(
        "Vec<Box<i32>>: {:?}",
        vec![Box::new(-1i32), Box::new(2), Box::new(3), Box::new(4)]
    );
    defmt::info!("Box<Vec<i32>>: {:?}", Box::new(vec![-1i32, 2, 3, 4]));
    defmt::info!(
        "String: {:?}",
        String::from("Hello! I'm a heap-allocated String")
    );

    loop {
        debug::exit(debug::EXIT_SUCCESS)
    }
}

#[derive(Format)]
struct NestedStruct {
    a: u8,
    b: u32,
}

fn nested() -> NestedStruct {
    defmt::info!("in nested {:u8}", 123);
    NestedStruct {
        a: 0xAA,
        b: 0x12345678,
    }
}

#[defmt::timestamp]
fn timestamp() -> u64 {
    // monotonic counter
    static COUNT: AtomicU32 = AtomicU32::new(0);
    COUNT.fetch_add(1, Ordering::Relaxed) as u64
}

mod if_alloc {
    use alloc_cortex_m::CortexMHeap;
    use core::alloc::Layout;

    #[global_allocator]
    static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

    #[alloc_error_handler]
    fn oom(_: Layout) -> ! {
        loop {}
    }

    pub fn init() {
        // Initialize the allocator BEFORE you use it
        let start = cortex_m_rt::heap_start() as usize;
        let size = 1024; // in bytes
        unsafe { ALLOCATOR.init(start, size) }
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
