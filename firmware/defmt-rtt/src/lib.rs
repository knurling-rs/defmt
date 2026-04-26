//! [`defmt`](https://github.com/knurling-rs/defmt) global logger over RTT.
//!
//! NOTE when using this crate it's not possible to use (link to) the
//! `rtt-target` crate
//!
//! To use this crate, link to it by importing it somewhere in your project.
//!
//! ```
//! // src/main.rs or src/bin/my-app.rs
//! use defmt_rtt as _;
//! ```
//!
//! # Blocking/Non-blocking
//!
//! `probe-rs` puts RTT into blocking-mode, to avoid losing data.
//!
//! As an effect this implementation may block forever if `probe-rs` disconnects
//! at runtime. This is because the RTT buffer will fill up and writing will
//! eventually halt the program execution.
//!
//! `defmt::flush` would also block forever in that case.
//!
//! If losing data is not an concern you can disable blocking mode by enabling
//! the feature `disable-blocking-mode`
//!
//! # Critical section implementation
//!
//! This crate uses
//! [`critical-section`](https://github.com/rust-embedded/critical-section) to
//! ensure only one thread is writing to the buffer at a time. You must import a
//! crate that provides a `critical-section` implementation suitable for the
//! current target. See the `critical-section` README for details.
//!
//! For example, for single-core privileged-mode Cortex-M targets, you can add
//! the following to your Cargo.toml.
//!
//! ```toml
//! [dependencies]
//! cortex-m = { version = "0.7.6", features = ["critical-section-single-core"]}
//! ```
//!
//! With feature `disable-irq-masking` you do not need a critical section
//! implementation and interrupts are not disabled. Instead, when execution
//! contexts collide, frames are dropped. This mode is for bare-metal
//! Cortex-M use where thread mode is a single execution context. It is not
//! correct on RTOS or other multi-thread-mode systems because all thread-mode
//! tasks share `IPSR == 0`, which can misidentify ownership and panic. It can
//! be combined with `disable-blocking-mode`, in which case frames that do not
//! already fit are dropped. Because the public write cursor advances only once
//! at frame end, every encoded frame in this mode must fit within the RTT
//! ring's usable capacity (`BUF_SIZE - 1`); oversized frames are dropped even
//! in blocking mode.

#![no_std]

mod channel;
mod consts;

use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicU32, Ordering},
};

#[cfg(not(feature = "disable-irq-masking"))]
use core::sync::atomic::AtomicBool;

use crate::{channel::Channel, consts::BUF_SIZE};

/// The relevant bits in the mode field in the Header
const MODE_MASK: u32 = 0b11;

/// Block the application if the RTT buffer is full, wait for the host to read data.
const MODE_BLOCK_IF_FULL: u32 = 2;

/// Don't block if the RTT buffer is full. Truncate data to output as much as fits.
const MODE_NON_BLOCKING_TRIM: u32 = 1;

/// The defmt global logger
///
/// The defmt crate requires that this be a unit type, so our state is stored in
/// [`RTT_ENCODER`] instead.
#[defmt::global_logger]
struct Logger;

/// Our defmt encoder state
#[cfg(not(feature = "disable-irq-masking"))]
static RTT_ENCODER: RttEncoder = RttEncoder::new();
#[cfg(feature = "disable-irq-masking")]
static RTT_ENCODER: AtomicRttEncoder = AtomicRttEncoder::new();

/// Our shared header structure.
///
/// The host will read this structure so it must be arranged as expected.
///
/// NOTE the `rtt-target` API is too permissive. It allows writing arbitrary
/// data to any channel (`set_print_channel` + `rprint*`) and that can corrupt
/// defmt log frames. So we declare the RTT control block here and make it
/// impossible to use `rtt-target` together with this crate.
#[no_mangle]
static _SEGGER_RTT: Header = Header {
    id: *b"SEGGER RTT\0\0\0\0\0\0",
    max_up_channels: 1,
    max_down_channels: 0,
    up_channel: Channel {
        name: NAME.as_ptr(),
        buffer: BUFFER.get(),
        size: BUF_SIZE as u32,
        write: AtomicU32::new(0),
        read: AtomicU32::new(0),
        flags: AtomicU32::new(MODE_NON_BLOCKING_TRIM),
    },
};

/// Report whether the SEGGER RTT up channel is in blocking mode.
///
/// Returns true if the mode bitfield within the flags value has been set to
/// `SEGGER_RTT_MODE_BLOCK_IF_FIFO_FULL`.
///
/// Currently we start-up in non-blocking mode, so if it's been set to blocking
/// mode then the connected client (e.g. probe-rs) must have done it.
pub fn in_blocking_mode() -> bool {
    (_SEGGER_RTT.up_channel.flags.load(Ordering::Relaxed) & MODE_MASK) == MODE_BLOCK_IF_FULL
}

/// Our shared buffer
#[cfg_attr(target_os = "macos", link_section = ".uninit,defmt-rtt.BUFFER")]
#[cfg_attr(not(target_os = "macos"), link_section = ".uninit.defmt-rtt.BUFFER")]
static BUFFER: Buffer = Buffer::new();

/// The name of our channel.
///
/// This is in a data section, so the whole RTT header can be read from RAM.
/// This is useful if flash access gets disabled by the firmware at runtime.
#[cfg_attr(target_os = "macos", link_section = ".data,defmt-rtt.NAME")]
#[cfg_attr(not(target_os = "macos"), link_section = ".data.defmt-rtt.NAME")]
static NAME: [u8; 6] = *b"defmt\0";

#[cfg(not(feature = "disable-irq-masking"))]
struct RttEncoder {
    /// A boolean lock
    ///
    /// Is `true` when `acquire` has been called and we have exclusive access to
    /// the rest of this structure.
    taken: AtomicBool,
    /// We need to remember this to exit a critical section
    cs_restore: UnsafeCell<critical_section::RestoreState>,
    /// A defmt::Encoder for encoding frames
    encoder: UnsafeCell<defmt::Encoder>,
}

#[cfg(feature = "disable-irq-masking")]
const NO_OWNER: u32 = u32::MAX;

#[cfg(feature = "disable-irq-masking")]
struct AtomicRttEncoder {
    /// A defmt::Encoder for encoding frames
    encoder: UnsafeCell<defmt::Encoder>,
    owner: AtomicU32,
    overflowed: UnsafeCell<bool>,
    start: UnsafeCell<usize>,
    cursor: UnsafeCell<usize>,
}

#[cfg(not(feature = "disable-irq-masking"))]
impl RttEncoder {
    /// Create a new semihosting-based defmt-encoder
    const fn new() -> RttEncoder {
        RttEncoder {
            taken: AtomicBool::new(false),
            cs_restore: UnsafeCell::new(critical_section::RestoreState::invalid()),
            encoder: UnsafeCell::new(defmt::Encoder::new()),
        }
    }

    /// Acquire the defmt encoder.
    fn acquire(&self) {
        // safety: Must be paired with corresponding call to release(), see below
        let restore = unsafe { critical_section::acquire() };

        // NB: You can re-enter critical sections but we need to make sure
        // no-one does that.
        if self.taken.load(Ordering::Relaxed) {
            panic!("defmt logger taken reentrantly")
        }

        // no need for CAS because we are in a critical section
        self.taken.store(true, Ordering::Relaxed);

        // safety: accessing the cell is OK because we have acquired a critical
        // section.
        unsafe {
            self.cs_restore.get().write(restore);
            let encoder: &mut defmt::Encoder = &mut *self.encoder.get();
            encoder.start_frame(|b| {
                _SEGGER_RTT.up_channel.write_all(b);
            });
        }
    }

    /// Write bytes to the defmt encoder.
    ///
    /// # Safety
    ///
    /// Do not call unless you have called `acquire`.
    unsafe fn write(&self, bytes: &[u8]) {
        // safety: accessing the cell is OK because we have acquired a critical
        // section.
        unsafe {
            let encoder: &mut defmt::Encoder = &mut *self.encoder.get();
            encoder.write(bytes, |b| {
                _SEGGER_RTT.up_channel.write_all(b);
            });
        }
    }

    /// Flush the encoder
    ///
    /// # Safety
    ///
    /// Do not call unless you have called `acquire`.
    unsafe fn flush(&self) {
        // safety: accessing the `&'static _` is OK because we have acquired a
        // critical section.
        _SEGGER_RTT.up_channel.flush();
    }

    /// Release the defmt encoder.
    ///
    /// # Safety
    ///
    /// Do not call unless you have called `acquire`. This will release
    /// your lock - do not call `flush` and `write` until you have done another
    /// `acquire`.
    unsafe fn release(&self) {
        if !self.taken.load(Ordering::Relaxed) {
            panic!("defmt release out of context")
        }

        // safety: accessing the cell is OK because we have acquired a critical
        // section.
        unsafe {
            let encoder: &mut defmt::Encoder = &mut *self.encoder.get();
            encoder.end_frame(|b| {
                _SEGGER_RTT.up_channel.write_all(b);
            });
            let restore = self.cs_restore.get().read();
            self.taken.store(false, Ordering::Relaxed);
            // paired with exactly one acquire call
            critical_section::release(restore);
        }
    }
}

#[cfg(feature = "disable-irq-masking")]
impl AtomicRttEncoder {
    /// Create a new semihosting-based defmt-encoder
    const fn new() -> AtomicRttEncoder {
        AtomicRttEncoder {
            encoder: UnsafeCell::new(defmt::Encoder::new()),
            owner: AtomicU32::new(NO_OWNER),
            overflowed: UnsafeCell::new(false),
            start: UnsafeCell::new(0),
            cursor: UnsafeCell::new(0),
        }
    }

    fn current_context() -> u32 {
        #[cfg(target_arch = "arm")]
        unsafe {
            // IPSR is 0 in thread mode and otherwise the active exception
            // number. That makes it a unique owner token for bare-metal
            // Cortex-M execution: one thread-mode context plus interrupts. It
            // is not suitable for systems that can switch between multiple
            // thread-mode tasks mid-frame because those tasks all appear as
            // owner 0 and can spuriously panic as "reentrant".
            let ipsr: u32;
            core::arch::asm!(
                "mrs {}, ipsr",
                out(reg) ipsr,
                options(nomem, nostack, preserves_flags)
            );
            ipsr
        }

        #[cfg(not(target_arch = "arm"))]
        0
    }

    fn acquire(&self) {
        let context = Self::current_context();
        if self.owner.load(Ordering::Relaxed) == context {
            panic!("defmt logger taken reentrantly");
        }

        // Acquire publishes exclusive ownership before any frame staging starts.
        // On failure we intentionally drop the whole colliding frame; the
        // caller continues but all later methods become no-ops because it is
        // not the owner.
        if self
            .owner
            .compare_exchange(NO_OWNER, context, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return;
        }

        unsafe {
            *self.overflowed.get() = false;
            let cursor = _SEGGER_RTT.up_channel.write.load(Ordering::Acquire) as usize;
            *self.start.get() = cursor;
            *self.cursor.get() = cursor;
            let encoder: &mut defmt::Encoder = &mut *self.encoder.get();
            encoder.start_frame(stage_bytes);
        }
    }

    /// Write bytes to the defmt encoder.
    ///
    /// # Safety
    ///
    /// Do not call unless you have called `acquire`.
    unsafe fn write(&self, bytes: &[u8]) {
        if self.owner.load(Ordering::Relaxed) != Self::current_context() {
            return;
        }
        unsafe {
            let encoder: &mut defmt::Encoder = &mut *self.encoder.get();
            encoder.write(bytes, stage_bytes);
        }
    }

    /// Flush the encoder.
    ///
    /// # Safety
    ///
    /// Do not call unless you have called `acquire`.
    unsafe fn flush(&self) {
        _SEGGER_RTT.up_channel.flush();
    }

    /// Release the defmt encoder.
    ///
    /// # Safety
    ///
    /// Do not call unless you have called `acquire`. This will release your
    /// lock - do not call `flush` and `write` until you have done another
    /// `acquire`.
    unsafe fn release(&self) {
        if self.owner.load(Ordering::Relaxed) != Self::current_context() {
            return;
        }
        unsafe {
            let encoder: &mut defmt::Encoder = &mut *self.encoder.get();
            encoder.end_frame(stage_bytes);
            if !*self.overflowed.get() {
                _SEGGER_RTT.up_channel.commit(*self.cursor.get());
            }
        }

        self.owner.store(NO_OWNER, Ordering::Release);
    }

    unsafe fn stage(&self, bytes: &[u8]) {
        unsafe {
            if *self.overflowed.get() {
                return;
            }
            if crate::channel::available_buffer_size(*self.start.get(), *self.cursor.get())
                < bytes.len()
            {
                *self.overflowed.get() = true;
                return;
            }

            // Only the owner stages bytes. Until `release()` commits the final
            // cursor, the host still sees the old write pointer and therefore
            // never consumes these bytes as a partial frame.
            if !_SEGGER_RTT
                .up_channel
                .stage_bytes(&mut *self.cursor.get(), bytes)
            {
                *self.overflowed.get() = true;
            }
        }
    }
}

#[cfg(not(feature = "disable-irq-masking"))]
unsafe impl Sync for RttEncoder {}
#[cfg(feature = "disable-irq-masking")]
unsafe impl Sync for AtomicRttEncoder {}

#[cfg(feature = "disable-irq-masking")]
fn stage_bytes(bytes: &[u8]) {
    unsafe {
        RTT_ENCODER.stage(bytes);
    }
}

unsafe impl defmt::Logger for Logger {
    fn acquire() {
        RTT_ENCODER.acquire();
    }

    unsafe fn write(bytes: &[u8]) {
        unsafe {
            RTT_ENCODER.write(bytes);
        }
    }

    unsafe fn flush() {
        unsafe {
            RTT_ENCODER.flush();
        }
    }

    unsafe fn release() {
        unsafe {
            RTT_ENCODER.release();
        }
    }
}

#[repr(C)]
struct Header {
    id: [u8; 16],
    max_up_channels: u32,
    max_down_channels: u32,
    up_channel: Channel,
}

unsafe impl Sync for Header {}

struct Buffer {
    inner: UnsafeCell<[u8; BUF_SIZE]>,
}

impl Buffer {
    const fn new() -> Buffer {
        Buffer {
            inner: UnsafeCell::new([0; BUF_SIZE]),
        }
    }

    const fn get(&self) -> *mut u8 {
        self.inner.get() as _
    }
}

unsafe impl Sync for Buffer {}
