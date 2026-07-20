//! [`defmt`](https://github.com/knurling-rs/defmt) global logger for programs that run on a
//! host operating system (Linux, macOS, ...) rather than on bare metal.
//!
//! The binary defmt wire data is written to stdout, or to the file named by the
//! `DEFMT_STDOUT_FILE` environment variable if it is set. Pipe the output through
//! `defmt-print` to view the logs:
//!
//! ```console
//! $ cargo run | defmt-print -e target/debug/my-app
//! ```
//!
//! To use this crate, link to it by importing it somewhere in your project.
//!
//! ```
//! // src/main.rs or src/bin/my-app.rs
//! use defmt_stdout as _;
//! ```
//!
//! # Timestamps and panics
//!
//! On bare-metal targets the `defmt.x` linker script provides default implementations of the
//! `_defmt_timestamp` and `_defmt_panic` symbols. Host programs are linked without that
//! linker script, so this crate provides those defaults instead:
//!
//! * the `timestamp` feature (enabled by default) defines an empty timestamp. Disable it if
//!   your application defines its own with [`defmt::timestamp!`].
//! * the `panic-handler` feature (enabled by default) defines a `_defmt_panic` that forwards
//!   to [`core::panic!`]. Disable it if your application defines its own with
//!   `#[defmt::panic_handler]`.
//!
//! # Critical section implementation
//!
//! This crate uses [`critical-section`](https://github.com/rust-embedded/critical-section)
//! with its `std` implementation to ensure only one thread is writing at a time.

use std::{
    cell::UnsafeCell,
    fs::File,
    io::{self, Write},
    sync::atomic::{AtomicBool, Ordering},
};

/// Name of the environment variable that redirects the log output to a file.
///
/// If it is not set, logs are written to stdout. The variable is read once, when the first
/// log frame is written; the file is created (truncated) at that point.
pub const FILE_ENV_VAR: &str = "DEFMT_STDOUT_FILE";

/// The defmt global logger
///
/// The defmt crate requires that this be a unit type, so our state is stored in
/// [`HOST_ENCODER`] instead.
#[defmt::global_logger]
struct Logger;

/// Our defmt encoder state
static HOST_ENCODER: HostEncoder = HostEncoder::new();

/// Where the encoded bytes are sent
enum Sink {
    Stdout(io::Stdout),
    File(File),
}

impl Sink {
    /// Select the sink based on the environment.
    fn from_env() -> Sink {
        match std::env::var_os(FILE_ENV_VAR) {
            Some(path) => match File::create(&path) {
                Ok(file) => Sink::File(file),
                Err(e) => panic!(
                    "defmt-stdout: failed to create `{}` (from {FILE_ENV_VAR}): {e}",
                    path.to_string_lossy()
                ),
            },
            None => Sink::Stdout(io::stdout()),
        }
    }

    /// Write bytes to the sink.
    ///
    /// Logging must not panic, so I/O errors are discarded.
    fn write_all(&mut self, bytes: &[u8]) {
        let _ = match self {
            Sink::Stdout(stdout) => stdout.write_all(bytes),
            Sink::File(file) => file.write_all(bytes),
        };
    }

    /// Flush the sink.
    fn flush(&mut self) {
        let _ = match self {
            Sink::Stdout(stdout) => stdout.flush(),
            Sink::File(file) => file.flush(),
        };
    }
}

struct HostEncoder {
    /// A boolean lock
    ///
    /// Is `true` when `acquire` has been called and we have exclusive access to
    /// the rest of this structure.
    taken: AtomicBool,
    /// We need to remember this to exit a critical section
    cs_restore: UnsafeCell<critical_section::RestoreState>,
    /// A defmt::Encoder for encoding frames
    encoder: UnsafeCell<defmt::Encoder>,
    /// Where the encoded bytes go; lazily selected when the first frame is written
    sink: UnsafeCell<Option<Sink>>,
}

impl HostEncoder {
    /// Create a new stdout-based defmt-encoder
    const fn new() -> HostEncoder {
        HostEncoder {
            taken: AtomicBool::new(false),
            cs_restore: UnsafeCell::new(critical_section::RestoreState::invalid()),
            encoder: UnsafeCell::new(defmt::Encoder::new()),
            sink: UnsafeCell::new(None),
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

        // safety: accessing the cells is OK because we have acquired a critical
        // section.
        unsafe {
            self.cs_restore.get().write(restore);
            let sink = (*self.sink.get()).get_or_insert_with(Sink::from_env);
            let encoder: &mut defmt::Encoder = &mut *self.encoder.get();
            encoder.start_frame(|b| sink.write_all(b));
        }
    }

    /// Write bytes to the defmt encoder.
    ///
    /// # Safety
    ///
    /// Do not call unless you have called `acquire`.
    unsafe fn write(&self, bytes: &[u8]) {
        // safety: accessing the cells is OK because we have acquired a critical
        // section.
        unsafe {
            let sink = (*self.sink.get()).get_or_insert_with(Sink::from_env);
            let encoder: &mut defmt::Encoder = &mut *self.encoder.get();
            encoder.write(bytes, |b| sink.write_all(b));
        }
    }

    /// Flush the sink
    ///
    /// # Safety
    ///
    /// Do not call unless you have called `acquire`.
    unsafe fn flush(&self) {
        // safety: accessing the cell is OK because we have acquired a critical
        // section.
        unsafe {
            if let Some(sink) = &mut *self.sink.get() {
                sink.flush();
            }
        }
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

        // safety: accessing the cells is OK because we have acquired a critical
        // section.
        unsafe {
            let sink = (*self.sink.get()).get_or_insert_with(Sink::from_env);
            let encoder: &mut defmt::Encoder = &mut *self.encoder.get();
            encoder.end_frame(|b| sink.write_all(b));
            // flush at the end of each frame so that readers on the other side of a
            // pipe (e.g. `defmt-print`) see complete frames promptly
            sink.flush();
            let restore = self.cs_restore.get().read();
            self.taken.store(false, Ordering::Relaxed);
            // paired with exactly one acquire call
            critical_section::release(restore);
        }
    }
}

unsafe impl Sync for HostEncoder {}

unsafe impl defmt::Logger for Logger {
    fn acquire() {
        HOST_ENCODER.acquire();
    }

    unsafe fn write(bytes: &[u8]) {
        unsafe {
            HOST_ENCODER.write(bytes);
        }
    }

    unsafe fn flush() {
        unsafe {
            HOST_ENCODER.flush();
        }
    }

    unsafe fn release() {
        unsafe {
            HOST_ENCODER.release();
        }
    }
}

/// An empty timestamp.
///
/// On bare-metal targets this default is provided by the `defmt.x` linker script; host
/// programs don't use that linker script so we provide it here instead. Disable the
/// `timestamp` feature to define your own with `defmt::timestamp!`.
#[cfg(feature = "timestamp")]
#[export_name = "_defmt_timestamp"]
fn defmt_timestamp(_: defmt::Formatter<'_>) {}

/// Forward `defmt::panic!` and friends to `core::panic!`.
///
/// The panic message has already been logged via defmt when this is called. Disable the
/// `panic-handler` feature to define your own with `#[defmt::panic_handler]`.
#[cfg(feature = "panic-handler")]
#[export_name = "_defmt_panic"]
fn defmt_panic() -> ! {
    panic!("panicked via defmt (the actual panic message was logged through defmt)")
}
