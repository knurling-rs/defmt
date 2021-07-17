#[cfg(all(feature = "encoding-raw", feature = "encoding-rzcrawobs"))]
compile_error!("Multiple `encoding-*` features are enabled. You may only enable one.");

#[cfg_attr(feature = "encoding-raw", path = "raw.rs")]
#[cfg_attr(not(feature = "encoding-raw"), path = "rzcobs.rs")]
mod inner;

// This wrapper struct is to avoid copypasting the public docs in all the impls.

/// TODO
pub struct Encoder {
    inner: inner::Encoder,
}

impl Encoder {
    /// TODO
    pub const fn new() -> Self {
        Self {
            inner: inner::Encoder::new(),
        }
    }

    /// TODO
    pub fn start_frame(&mut self, write: impl FnMut(&[u8])) {
        self.inner.start_frame(write)
    }

    /// TODO
    pub fn end_frame(&mut self, write: impl FnMut(&[u8])) {
        self.inner.end_frame(write)
    }

    /// TODO
    pub fn write(&mut self, data: &[u8], write: impl FnMut(&[u8])) {
        self.inner.write(data, write)
    }
}
