use super::*;

impl<T> Format for alloc::boxed::Box<T>
where
    T: ?Sized + Format,
{
    fn format(&self, f: Formatter) {
        T::format(&*self, f)
    }
}

impl<T> Format for alloc::rc::Rc<T>
where
    T: ?Sized + Format,
{
    fn format(&self, f: Formatter) {
        T::format(&*self, f)
    }
}

#[cfg(not(no_cas))]
impl<T> Format for alloc::sync::Arc<T>
where
    T: ?Sized + Format,
{
    fn format(&self, f: Formatter) {
        T::format(&*self, f)
    }
}

impl<T> Format for alloc::vec::Vec<T>
where
    T: Format,
{
    fn format(&self, f: Formatter) {
        self.as_slice().format(f)
    }
}

impl Format for alloc::string::String {
    fn format(&self, f: Formatter) {
        self.as_str().format(f)
    }
}

impl<'a, T> Format for alloc::borrow::Cow<'a, [T]>
where
    T: 'a + Format,
    [T]: alloc::borrow::ToOwned<Owned = alloc::vec::Vec<T>>,
{
    fn format(&self, f: Formatter) {
        self.as_ref().format(f)
    }
}

impl<'a> Format for alloc::borrow::Cow<'a, str> {
    fn format(&self, f: Formatter) {
        self.as_ref().format(f)
    }
}
