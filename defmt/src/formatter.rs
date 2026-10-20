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
    /// 16-bit address
    pub(crate) address: u16,
}

impl Str {
    /// The index of this string in the `.defmt` table of the firmware's ELF.
    ///
    /// Lets a program send an interned string over a channel other than the defmt logger (its own protocol, a
    /// register, a status frame) as two bytes; the host looks it up with `defmt_decoder::Table::get_str`. The index
    /// is valid for the ELF it was built in.
    pub const fn index(&self) -> u16 {
        self.address
    }
}
