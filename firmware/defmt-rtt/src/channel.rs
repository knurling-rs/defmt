use core::{
    ptr,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::{consts::BUF_SIZE, MODE_BLOCK_IF_FULL, MODE_MASK};

/// RTT Up channel
#[repr(C)]
pub(crate) struct Channel {
    /// Name of the channel (null terminated)
    pub name: *const u8,
    /// Pointer to the RTT buffer.
    pub buffer: *mut u8,
    /// Size, in bytes, of the RTT buffer
    pub size: u32,
    /// Written by the target.
    pub write: AtomicU32,
    /// Written by the host.
    pub read: AtomicU32,
    /// Channel properties.
    ///
    /// Currently, only the lowest 2 bits are used to set the channel mode (see constants below).
    pub flags: AtomicU32,
}

impl Channel {
    pub fn write_all(&self, mut bytes: &[u8]) {
        // the host-connection-status is only modified after RAM initialization while the device is
        // halted, so we only need to check it once before the write-loop
        let write = match self.host_is_connected() {
            _ if cfg!(feature = "disable-blocking-mode") => Self::nonblocking_write,
            true => Self::blocking_write,
            false => Self::nonblocking_write,
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
        let read = self.read.load(Ordering::Relaxed) as usize;
        let write = self.write.load(Ordering::Acquire) as usize;
        let available = available_buffer_size(read, write);

        // abort if buffer is full
        if available == 0 {
            return 0;
        }

        self.write_impl(bytes, write, available)
    }

    fn nonblocking_write(&self, bytes: &[u8]) -> usize {
        let write = self.write.load(Ordering::Acquire) as usize;

        // NOTE truncate at BUF_SIZE to avoid more than one "wrap-around" in a single `write` call
        self.write_impl(bytes, write, BUF_SIZE)
    }

    fn write_impl(&self, bytes: &[u8], cursor: usize, available: usize) -> usize {
        let len = bytes.len().min(available);

        // copy `bytes[..len]` to the RTT buffer
        unsafe {
            if cursor + len > BUF_SIZE {
                // split memcpy
                let pivot = BUF_SIZE - cursor;
                ptr::copy_nonoverlapping(bytes.as_ptr(), self.buffer.add(cursor), pivot);
                ptr::copy_nonoverlapping(bytes.as_ptr().add(pivot), self.buffer, len - pivot);
            } else {
                // single memcpy
                ptr::copy_nonoverlapping(bytes.as_ptr(), self.buffer.add(cursor), len);
            }
        }

        // adjust the write pointer, so the host knows that there is new data
        self.write.store(
            (cursor.wrapping_add(len) % BUF_SIZE) as u32,
            Ordering::Release,
        );

        // return the number of bytes written
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
fn available_buffer_size(read_cursor: usize, write_cursor: usize) -> usize {
    if read_cursor > write_cursor {
        read_cursor - write_cursor - 1
    } else {
        BUF_SIZE - write_cursor - 1 + read_cursor
    }
}

#[cfg(test)]
mod tests {
    use super::available_buffer_size;
    use crate::consts::BUF_SIZE;

    #[test]
    fn test_rtt_available_buffer_size() {
        // Helper to simulate RTT buffer state
        let avail = |read: usize, write: usize| available_buffer_size(read, write);

        // --- Case 1: Buffer is EMPTY (write == read) ---
        // Should have maximum available space: BUF_SIZE - 1
        assert_eq!(avail(0, 0), BUF_SIZE - 1);
        assert_eq!(avail(10, 10), BUF_SIZE - 1);
        assert_eq!(avail(BUF_SIZE - 1, BUF_SIZE - 1), BUF_SIZE - 1);

        // --- Case 2: Buffer is FULL ---
        // Full condition: (write + 1) % BUF_SIZE == read
        // i.e., write == (read - 1 + BUF_SIZE) % BUF_SIZE
        assert_eq!(avail(0, BUF_SIZE - 1), 0); // write=BUF_SIZE-1, read=0 → full
        assert_eq!(avail(5, 4), 0); // write=4, read=5 → full
        assert_eq!(avail(1, 0), 0); // write=0, read=1 → full

        // --- Case 3: Read ahead of Write (no wrap-around) ---
        // e.g., read=10, write=5 → free space = [5..9] → size = 10 - 5 - 1 = 4
        assert_eq!(avail(10, 5), 10 - 5 - 1);
        assert_eq!(avail(BUF_SIZE - 1, 0), (BUF_SIZE - 1) - 0 - 1); // = BUF_SIZE - 2

        // --- Case 4: Write has wrapped around, Read behind (wrap-around case) ---
        // e.g., read=5, write=10 → free space = [10..BUF_SIZE-1] + [0..4]
        // size = (BUF_SIZE - 10 - 1) + (5) = BUF_SIZE - 10 - 1 + 5
        assert_eq!(avail(5, 10), BUF_SIZE - 10 - 1 + 5);
        assert_eq!(avail(0, 1), BUF_SIZE - 1 - 1 + 0); // = BUF_SIZE - 2
        assert_eq!(avail(1, BUF_SIZE - 1), BUF_SIZE - (BUF_SIZE - 1) - 1 + 1); // = 1

        // --- Edge: Single byte free ---
        // After filling BUF_SIZE - 2 bytes from empty, 1 byte remains
        assert_eq!(avail(1, BUF_SIZE - 1), 1); // one byte free
        assert_eq!(avail(2, BUF_SIZE - 1), 2); // one byte free: only position BUF_SIZE-1 is free? No.
                                               // Actually: write=BUF_SIZE-1, read=2 → free = [BUF_SIZE-1] + [0,1] → but [0,1] is 2 bytes?
                                               // Let's recompute: total free = (BUF_SIZE - (BUF_SIZE-1) - 1) + 2 = (0) + 2 = 2 → wait.

        // Better: use invariant
        // Total data in buffer = (write - read + BUF_SIZE) % BUF_SIZE
        // Free = BUF_SIZE - 1 - data
        let data_in_buffer = |read: usize, write: usize| (write + BUF_SIZE - read) % BUF_SIZE;
        let free_should_be = |read: usize, write: usize| BUF_SIZE - 1 - data_in_buffer(read, write);

        // Validate our function against this invariant
        for read in 0..BUF_SIZE.min(64) {
            for write in 0..BUF_SIZE.min(64) {
                let expected = free_should_be(read, write);
                let actual = avail(read, write);
                assert_eq!(actual, expected, "Mismatch at read={read}, write={write}");
            }
        }
    }
}
