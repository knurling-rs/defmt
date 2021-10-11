use core::{
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{MODE_BLOCK_IF_FULL, MODE_MASK, SIZE};

/// RTT Up channel
#[repr(C)]
pub(crate) struct Channel {
    pub name: *const u8,
    /// Pointer to the RTT buffer.
    pub buffer: *mut u8,
    pub size: usize,
    /// Written by the target.
    pub write: AtomicUsize,
    /// Written by the host.
    pub read: AtomicUsize,
    /// Channel properties.
    ///
    /// Currently, only the lowest 2 bits are used to set the channel mode (see constants below).
    pub flags: AtomicUsize,
}

impl Channel {
    pub fn write_all(&self, mut bytes: &[u8]) {
        // the host-connection-status is only modified after RAM initialization while the device is
        // halted, so we only need to check it once before the write-loop
        let write = match self.host_is_connected() {
            true => Channel::blocking_write,
            false => Channel::nonblocking_write,
        };

        while !bytes.is_empty() {
            let consumed = write(self, bytes);
            if consumed != 0 {
                bytes = &bytes[consumed..];
            }
        }
    }

    fn blocking_write(&self, bytes: &[u8]) -> usize {
        if bytes.is_empty() {
            return 0;
        }

        // calculate how much space is left in the buffer
        let read = self.read.load(Ordering::Relaxed);
        let write = self.write.load(Ordering::Acquire);
        let available = available_buffer_size(read, write);
        // abort if buffer is full
        if available == 0 {
            return 0;
        }

        self.write_impl(bytes, available)
    }

    fn nonblocking_write(&self, bytes: &[u8]) -> usize {
        self.write_impl(bytes, SIZE)
    }

    /// Copy `available` (or all) data from `bytes` into the buffer.
    ///
    /// Returns the amount of bytes copied.
    fn write_impl(&self, bytes: &[u8], available: usize) -> usize {
        let cursor = self.write.load(Ordering::Acquire);
        let len = bytes.len().min(available);

        unsafe {
            if cursor + len > SIZE {
                // split memcpy
                let pivot = SIZE - cursor;
                ptr::copy_nonoverlapping(bytes.as_ptr(), self.buffer.add(cursor), pivot);
                ptr::copy_nonoverlapping(bytes.as_ptr().add(pivot), self.buffer, len - pivot);
            } else {
                // single memcpy
                ptr::copy_nonoverlapping(bytes.as_ptr(), self.buffer.add(cursor), len);
            }
        }
        self.write
            .store(cursor.wrapping_add(len) % SIZE, Ordering::Release);

        len
    }

    pub fn flush(&self) {
        // return early, if host is disconnected
        if !self.host_is_connected() {
            return;
        }

        // busy wait, until the read- catches up with the write-pointer
        let read = || self.read.load(Ordering::Relaxed);
        let write = || self.write.load(Ordering::Relaxed);
        while read() != write() {}
    }

    fn host_is_connected(&self) -> bool {
        // we assume that a host is connected if we are in blocking-mode. this is what probe-run does.
        self.flags.load(Ordering::Relaxed) & MODE_MASK == MODE_BLOCK_IF_FULL
    }
}

/// How much space is left in the buffer?
fn available_buffer_size(read: usize, write: usize) -> usize {
    match read > write {
        true => read - write - 1,
        false if read == 0 => SIZE - write - 1,
        false => SIZE - write,
    }
}
