#[cfg(feature = "alloc")]
mod alloc_;
mod arrays;
mod core_;
mod primitives;
mod tuples;

use defmt_macros::internp;

use crate::{self as defmt, Format, Formatter, Str};
