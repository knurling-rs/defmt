#![no_std]

use core::{
    cmp, ptr,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use cortex_m::{interrupt, register};

// TODO make configurable
// NOTE use a power of 2 for best performance
const SIZE: usize = 1024;

struct Logger;

impl binfmt::Write for Logger {
    fn write(&mut self, bytes: &[u8]) {
        unsafe { handle().write_all(bytes) }
    }
}

static TAKEN: AtomicBool = AtomicBool::new(false);
static INTERRUPTS_ACTIVE: AtomicBool = AtomicBool::new(false);

#[no_mangle]
fn _binfmt_acquire() -> Option<binfmt::Formatter> {
    let primask = register::primask::read();
    interrupt::disable();
    if !TAKEN.load(Ordering::Relaxed) {
        // no need for CAS because interrupts are disabled
        TAKEN.store(true, Ordering::Relaxed);

        INTERRUPTS_ACTIVE.store(primask.is_active(), Ordering::Relaxed);

        Some(unsafe {
            binfmt::Formatter::from_raw(
                &Logger as &dyn binfmt::Write as *const dyn binfmt::Write as *mut dyn binfmt::Write,
            )
        })
    } else {
        if primask.is_active() {
            // re-enable interrupts
            unsafe { interrupt::enable() }
        }
        None
    }
}

#[no_mangle]
fn _binfmt_release(_: binfmt::Formatter) {
    TAKEN.store(false, Ordering::Relaxed);
    if INTERRUPTS_ACTIVE.load(Ordering::Relaxed) {
        // re-enable interrupts
        unsafe { interrupt::enable() }
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
    flags: usize,
}

impl Channel {
    fn write_all(&self, mut bytes: &[u8]) {
        while !bytes.is_empty() {
            let consumed = self.write(bytes);
            if consumed != 0 {
                bytes = &bytes[consumed..];
            }
        }
    }

    fn write(&self, bytes: &[u8]) -> usize {
        if bytes.is_empty() {
            return 0;
        }

        let read = self.write.load(Ordering::Relaxed);
        let write = self.write.load(Ordering::Acquire);
        let available = SIZE.wrapping_add(read).wrapping_sub(write);

        if available == 0 {
            return 0;
        }

        let cursor = write % SIZE;
        let len = cmp::min(bytes.len(), available);
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
        self.write.store(write.wrapping_add(len), Ordering::Release);
        len
    }
}

// make sure we only get shared references to the header/channel (avoid UB)
/// # Safety
/// `Channel` API is not re-entrant; this handle should not be held from different execution
/// contexts (e.g. thread-mode, interrupt context)
unsafe fn handle() -> &'static Channel {
    // NOTE the `rtt-target` API is too permissive. It allows writing arbitrary data to any
    // channel (`set_print_channel` + `rprint*`) and that can corrupt binfmt log frames.
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
            flags: 0b10, // mode = block-if-full
        },
    };

    #[link_section = ".uninit.binfmt-rtt.BUFFER"]
    static mut BUFFER: [u8; SIZE] = [0; SIZE];

    static NAME: &[u8] = b"binfmt\0";

    &_SEGGER_RTT.up_channel
}
