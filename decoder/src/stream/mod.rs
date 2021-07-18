mod raw;
mod rzcobs;

pub use raw::Raw;
pub use rzcobs::Rzcobs;

use crate::{DecodeError, Frame};

pub trait StreamDecoder {
    /// Push received data to the decoder. The decoder stores it
    /// internally, and makes decoded frames available through [`decode`].
    fn received(&mut self, data: &[u8]);

    fn decode(&mut self) -> Result<Frame<'_>, DecodeError>;
}
