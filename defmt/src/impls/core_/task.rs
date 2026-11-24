use super::*;

impl Format for core::task::Context<'_> {
    fn format(&self, fmt: Formatter) {
        crate::write!(fmt, "Context {{ waker: {} }})", self.waker())
    }
}

impl Format for core::task::Waker {
    fn format(&self, fmt: Formatter) {
        crate::write!(fmt, "Waker")
    }
}

impl Format for core::task::RawWakerVTable {
    fn format(&self, fmt: Formatter) {
        crate::write!(fmt, "RawWakerVTable")
    }
}

impl Format for core::task::RawWaker {
    fn format(&self, fmt: Formatter) {
        crate::write!(fmt, "RawWaker")
    }
}

impl<T> Format for core::task::Poll<T>
where
    T: Format,
{
    fn format(&self, fmt: Formatter) {
        match self {
            core::task::Poll::Ready(ready) => crate::write!(fmt, "Ready({})", ready),
            core::task::Poll::Pending => crate::write!(fmt, "Pending"),
        }
    }
}
