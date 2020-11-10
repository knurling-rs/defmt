//! [`defmt`](https://github.com/knurling-rs/defmt) global logger over RTT.
//!
//! NOTE when using this crate it's not possible to use (link to) the `rtt-target` crate
//!
//! To use this crate, link to it by importing it somewhere in your project.
//!
//! ```
//! // src/main.rs or src/bin/my-app.rs
//! use defmt_rtt as _;
//! ```

#![no_std]

use core::{
    ptr::{self, NonNull},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use cortex_m::{interrupt, register};

// TODO make configurable
// NOTE use a power of 2 for best performance
const SIZE: usize = 1024;

#[defmt::global_logger]
struct Logger;

impl defmt::Write for Logger {
    fn write(&mut self, bytes: &[u8]) {
        unsafe { handle().write_all(bytes) }
    }
}

static TAKEN: AtomicBool = AtomicBool::new(false);
static INTERRUPTS_ACTIVE: AtomicBool = AtomicBool::new(false);

unsafe impl defmt::Logger for Logger {
    fn acquire() -> Option<NonNull<dyn defmt::Write>> {
        let primask = register::primask::read();
        interrupt::disable();
        if !TAKEN.load(Ordering::Relaxed) {
            // no need for CAS because interrupts are disabled
            TAKEN.store(true, Ordering::Relaxed);

            INTERRUPTS_ACTIVE.store(primask.is_active(), Ordering::Relaxed);

            Some(NonNull::from(&Logger as &dyn defmt::Write))
        } else {
            if primask.is_active() {
                // re-enable interrupts
                unsafe { interrupt::enable() }
            }
            None
        }
    }

    unsafe fn release(_: NonNull<dyn defmt::Write>) {
        TAKEN.store(false, Ordering::Relaxed);
        if INTERRUPTS_ACTIVE.load(Ordering::Relaxed) {
            // re-enable interrupts
            interrupt::enable()
        }
    }
}

#[repr(C)]
struct Header {
    id: [u8; 16],
    max_up_channels: usize,
    max_down_channels: usize,
    up_channel: Channel,
}

#[repr(C)]
struct Channel {
    name: *const u8,
    buffer: *mut u8,
    size: usize,
    write: AtomicUsize,
    read: AtomicUsize,
    flags: AtomicUsize,
}

const BLOCK_IF_FULL: usize = 2;
const NOBLOCK_TRIM: usize = 1;

impl Channel {
    fn write_all(&self, mut bytes: &[u8]) {
        // NOTE `flags` is modified by the host after RAM initialization while the device is halted
        // it cannot otherwise be modified so we don't need to check its state more often than
        // just here
        if self.flags.load(Ordering::Relaxed) == BLOCK_IF_FULL {
            while !bytes.is_empty() {
                let consumed = self.blocking_write(bytes);
                if consumed != 0 {
                    bytes = &bytes[consumed..];
                }
            }
        } else {
            while !bytes.is_empty() {
                let consumed = self.nonblocking_write(bytes);
                if consumed != 0 {
                    bytes = &bytes[consumed..];
                }
            }
        }
    }

    fn blocking_write(&self, bytes: &[u8]) -> usize {
        if bytes.is_empty() {
            return 0;
        }

        let read = self.read.load(Ordering::Relaxed);
        let write = self.write.load(Ordering::Acquire);
        let available = if read > write {
            read - write - 1
        } else if read == 0 {
            SIZE - write - 1
        } else {
            SIZE - write
        };

        if available == 0 {
            return 0;
        }

        let cursor = write;
        let len = bytes.len().min(available);

        unsafe {
            if cursor + len > SIZE {
                // split memcpy
                let pivot = SIZE - cursor;
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    self.buffer.add(cursor.into()),
                    pivot.into(),
                );
                ptr::copy_nonoverlapping(
                    bytes.as_ptr().add(pivot.into()),
                    self.buffer,
                    (len - pivot).into(),
                );
            } else {
                // single memcpy
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    self.buffer.add(cursor.into()),
                    len.into(),
                );
            }
        }
        self.write
            .store(write.wrapping_add(len) % SIZE, Ordering::Release);

        len
    }

    fn nonblocking_write(&self, bytes: &[u8]) -> usize {
        let write = self.write.load(Ordering::Acquire);
        let cursor = write;
        // NOTE truncate at SIZE to avoid more than one "wrap-around" in a single `write` call
        let len = bytes.len().min(SIZE);

        unsafe {
            if cursor + len > SIZE {
                // split memcpy
                let pivot = SIZE - cursor;
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    self.buffer.add(cursor.into()),
                    pivot.into(),
                );
                ptr::copy_nonoverlapping(
                    bytes.as_ptr().add(pivot.into()),
                    self.buffer,
                    (len - pivot).into(),
                );
            } else {
                // single memcpy
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    self.buffer.add(cursor.into()),
                    len.into(),
                );
            }
        }
        self.write
            .store(write.wrapping_add(len) % SIZE, Ordering::Release);

        len
    }
}

// make sure we only get shared references to the header/channel (avoid UB)
/// # Safety
/// `Channel` API is not re-entrant; this handle should not be held from different execution
/// contexts (e.g. thread-mode, interrupt context)
unsafe fn handle() -> &'static Channel {
    // NOTE the `rtt-target` API is too permissive. It allows writing arbitrary data to any
    // channel (`set_print_channel` + `rprint*`) and that can corrupt defmt log frames.
    // So we declare the RTT control block here and make it impossible to use `rtt-target` together
    // with this crate.
    #[no_mangle]
    static mut _SEGGER_RTT: Header = Header {
        id: *b"SEGGER RTT\0\0\0\0\0\0",
        max_up_channels: 1,
        max_down_channels: 0,
        up_channel: Channel {
            name: NAME as *const _ as *const u8,
            buffer: unsafe { &mut BUFFER as *mut _ as *mut u8 },
            size: SIZE,
            write: AtomicUsize::new(0),
            read: AtomicUsize::new(0),
            flags: AtomicUsize::new(NOBLOCK_TRIM),
        },
    };

    #[link_section = ".uninit.defmt-rtt.BUFFER"]
    static mut BUFFER: [u8; SIZE] = [0; SIZE];

    static NAME: &[u8] = b"defmt\0";

    &_SEGGER_RTT.up_channel
}
