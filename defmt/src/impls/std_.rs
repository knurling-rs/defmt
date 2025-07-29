use super::*;
use std::sync::TryLockError;

impl<T> Format for std::sync::Mutex<T>
where
    T: ?Sized + Format,
{
    #[inline]
    fn format(&self, fmt: Formatter) {
        match self.try_lock() {
            Ok(guard) => crate::write!(fmt, "Mutex {{ data: {=?} }}", guard),
            Err(TryLockError::Poisoned(err)) => {
                crate::write!(fmt, "Mutex {{ data: {=?} }}", err.get_ref())
            }
            Err(TryLockError::WouldBlock) => crate::write!(fmt, "Mutex {{ data: <locked> }}"),
        }
    }
}

impl<T> Format for std::sync::MutexGuard<'_, T>
where
    T: ?Sized + Format,
{
    delegate_format!(T, self, &*self);
}

#[cfg(test)]
mod tests {
    use crate as defmt;
    use std::sync::Mutex;

    #[test]
    fn mutex() {
        let m = Mutex::new(42);
        defmt::info!("{}", m);
    }
}
