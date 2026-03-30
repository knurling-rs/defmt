use std::{
    cell::UnsafeCell,
    env::current_exe,
    fs,
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use defmt_decoder::Table;
use lazy_static::lazy_static;

#[defmt::global_logger]
struct Logger;

lazy_static! {
    static ref ENCODER: Mutex<StdoutLogger> = Mutex::new(StdoutLogger::new());
}

struct StdoutLogger {
    /// A boolean lock
    ///
    /// Is `true` when `acquire` has been called and we have exclusive access to
    /// the rest of this structure.
    taken: AtomicBool,
    /// A defmt::Encoder for encoding frames
    encoder: UnsafeCell<defmt::Encoder>,
    bytes: Vec<u8>,
    table: Table,
}

impl StdoutLogger {
    fn new() -> StdoutLogger {
        let bytes = fs::read(current_exe().unwrap()).unwrap();
        let table = defmt_decoder::Table::parse(&bytes)
            .unwrap()
            .expect(".defmt data not found");
        StdoutLogger {
            taken: AtomicBool::new(false),
            encoder: UnsafeCell::new(defmt::Encoder::new()),
            bytes: Vec::new(),
            table,
        }
    }

    fn acquire(&mut self) {
        if self.taken.load(Ordering::Relaxed) {
            panic!("defmt logger taken reentrantly")
        }
        self.taken.store(true, Ordering::Relaxed);

        unsafe {
            let encoder: &mut defmt::Encoder = &mut *self.encoder.get();
            encoder.start_frame(|b| {
                self.bytes.extend_from_slice(b);
            });
        }
    }

    unsafe fn write(&mut self, bytes: &[u8]) {
        unsafe {
            let encoder: &mut defmt::Encoder = &mut *self.encoder.get();
            // dbg!(bytes);
            encoder.write(bytes, |b| {
                // dbg!(b);
                self.bytes.extend_from_slice(b);
            });
        }
    }

    unsafe fn flush(&self) {}

    unsafe fn release(&mut self) {
        if !self.taken.load(Ordering::Relaxed) {
            panic!("defmt release out of context")
        }

        let encoder: &mut defmt::Encoder = unsafe { &mut *self.encoder.get() };
        encoder.end_frame(|b| {
            self.bytes.extend_from_slice(b);
        });
        // A frame just ended, self.bytes should now contain exactly one frame we can decode
        let mut decoder = self.table.new_stream_decoder();
        decoder.received(&self.bytes);
        self.bytes.clear();

        let res = decoder.decode();
        let frame = match res {
            Ok(frame) => frame,
            Err(error) => {
                eprintln!("{error}");
                return;
            }
        };
        println!("{}", frame.display(true));
        self.taken.store(false, Ordering::Relaxed);
    }
}

unsafe impl Sync for StdoutLogger {}

unsafe impl defmt::Logger for Logger {
    fn acquire() {
        ENCODER.lock().unwrap().acquire();
    }

    unsafe fn write(bytes: &[u8]) {
        unsafe {
            ENCODER.lock().unwrap().write(bytes);
        }
    }

    unsafe fn flush() {
        unsafe {
            ENCODER.lock().unwrap().flush();
        }
    }

    unsafe fn release() {
        unsafe {
            ENCODER.lock().unwrap().release();
        }
    }
}

static COUNT: AtomicUsize = AtomicUsize::new(0);
defmt::timestamp!("{=usize}", COUNT.fetch_add(1, Ordering::Relaxed));
