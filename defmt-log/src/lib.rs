//! An implementation of the defmt logging traits, but using the `log` crate for output

use std::sync::{Condvar, Mutex};

#[defmt::global_logger]
struct FakeLogger;

#[derive(PartialEq, Eq)]
enum State {
    Waiting,
    AcquiredNeedAddr,
    WantArgs,    
}

struct Context {
    state: State
}

impl Context {
    const fn new() -> Context {
        Context {
            state: State::Waiting
        }
    }
}

static CONTEXT: (Mutex<Context>, Condvar) = (Mutex::new(Context::new()), Condvar::new());

unsafe impl defmt::Logger for FakeLogger {
    fn acquire() {
        log::info!("Acquiring {:?}", std::thread::current().id());
        let mut ctx = CONTEXT.0.lock().unwrap();
        while ctx.state != State::Waiting {
            // sit on the condvar because only one thread can grab the lock
            ctx = CONTEXT.1.wait(ctx).unwrap();
        }
        // cool, we can take it
        ctx.state = State::AcquiredNeedAddr;
    }

    unsafe fn flush() {
        log::info!("Flushing {:?}", std::thread::current().id());
    }

    unsafe fn release() {
        log::info!("Releasing {:?}", std::thread::current().id());
        let mut ctx = CONTEXT.0.lock().unwrap();
        ctx.state = State::Waiting;
        CONTEXT.1.notify_one();
    }

    unsafe fn write(bytes: &[u8]) {
        use std::convert::TryInto;
        log::info!("Bytes {:?} {:02x?}", std::thread::current().id(), bytes);
        let mut ctx = CONTEXT.0.lock().unwrap();
        match ctx.state {
            State::Waiting => panic!("Unlocked write!!"),
            State::AcquiredNeedAddr => {
                let addr = &bytes[0..std::mem::size_of::<usize>()];
                let addr: usize = usize::from_le_bytes(addr.try_into().unwrap());
                let ptr = addr as *const &'static str;
                let format_str: &'static str = unsafe { ptr.read() };
                log::info!("Format string: {}", format_str);
                ctx.state = State::WantArgs;      
            },
            State::WantArgs => {
                log::info!("Arg: {:02x?}", bytes);
            },
        }
    }
}

#[export_name = "_defmt_timestamp"]
fn defmt_timestamp(_: defmt::Formatter<'_>) {}
