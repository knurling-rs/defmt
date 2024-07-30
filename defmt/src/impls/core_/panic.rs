use core::panic;

use super::*;

impl<'a> Format for panic::PanicInfo<'a> {
    fn format(&self, f: Formatter) {
        crate::write!(f, "panicked at {}", self.location());
        // TODO: consider supporting self.message() once stabilized, or add a crate feature for
        // conditional support
    }
}

impl<'a> Format for panic::Location<'a> {
    fn format(&self, f: Formatter) {
        crate::write!(
            f,
            "{=str}:{=u32}:{=u32}",
            self.file(),
            self.line(),
            self.column()
        );
    }
}
