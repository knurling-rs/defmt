use core::marker::PhantomData;

/// Handle to a defmt logger.
#[derive(Copy, Clone)]
pub struct Formatter<'a> {
    pub(crate) _phantom: PhantomData<&'a ()>,
}

/// An interned string created via [`intern!`].
///
/// [`intern!`]: macro.intern.html
#[derive(Clone, Copy)]
pub struct Str {
    /// The address of the interned string
    pub(crate) address: usize,
}
