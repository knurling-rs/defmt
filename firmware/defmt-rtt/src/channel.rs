use core::{
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{MODE_BLOCK_IF_FULL, MODE_MASK, SIZE};

#[repr(C)]
pub(crate) struct Channel {
    pub name: *const u8,
    pub buffer: *mut u8,
    pub size: usize,
    pub write: AtomicUsize,
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
        let write = match host_is_connected(self) {
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
                ptr::copy_nonoverlapping(bytes.as_ptr(), self.buffer.add(cursor), pivot);
                ptr::copy_nonoverlapping(bytes.as_ptr().add(pivot), self.buffer, len - pivot);
            } else {
                // single memcpy
                ptr::copy_nonoverlapping(bytes.as_ptr(), self.buffer.add(cursor), len);
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
                ptr::copy_nonoverlapping(bytes.as_ptr(), self.buffer.add(cursor), pivot);
                ptr::copy_nonoverlapping(bytes.as_ptr().add(pivot), self.buffer, len - pivot);
            } else {
                // single memcpy
                ptr::copy_nonoverlapping(bytes.as_ptr(), self.buffer.add(cursor), len);
            }
        }
        self.write
            .store(write.wrapping_add(len) % SIZE, Ordering::Release);

        len
    }

    pub fn flush(&self) {
        // return early, if host is disconnected
        if !host_is_connected(self) {
            return;
        }

        // busy wait, until the read- catches up with the write-pointer
        let read = || self.read.load(Ordering::Relaxed);
        let write = || self.write.load(Ordering::Relaxed);
        while read() != write() {}
    }
}

fn host_is_connected(channel: &Channel) -> bool {
    // we assume that a host is connected if we are in blocking-mode. this is what probe-run does.
    channel.flags.load(Ordering::Relaxed) & MODE_MASK == MODE_BLOCK_IF_FULL
}
