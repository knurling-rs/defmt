use super::*;

macro_rules! tuple {
    ( $format:expr, ($($name:ident),+) ) => (
        impl<$($name:Format),+> Format for ($($name,)+) where last_type!($($name,)+): ?Sized {
            #[allow(non_snake_case, unused_assignments)]
            fn format(&self, f: Formatter) {
                if f.inner.needs_tag() {
                    let t = internp!($format);
                    f.inner.u8(&t);
                }

                let ($(ref $name,)+) = *self;
                $(
                    let formatter = Formatter { inner: f.inner };
                    $name.format(formatter);
                )+
            }
        }
    )
}

macro_rules! last_type {
    ($a:ident,) => { $a };
    ($a:ident, $($rest_a:ident,)+) => { last_type!($($rest_a,)+) };
}

tuple! { "({=?})", (T0) }
tuple! { "({=?}, {=?})", (T0, T1) }
tuple! { "({=?}, {=?}, {=?})", (T0, T1, T2) }
tuple! { "({=?}, {=?}, {=?}, {=?})", (T0, T1, T2, T3) }
tuple! { "({=?}, {=?}, {=?}, {=?}, {=?})", (T0, T1, T2, T3, T4) }
tuple! { "({=?}, {=?}, {=?}, {=?}, {=?}, {=?})", (T0, T1, T2, T3, T4, T5) }
tuple! { "({=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?})", (T0, T1, T2, T3, T4, T5, T6) }
tuple! { "({=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?})", (T0, T1, T2, T3, T4, T5, T6, T7) }
tuple! { "({=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?})", (T0, T1, T2, T3, T4, T5, T6, T7, T8) }
tuple! { "({=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?})", (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9) }
tuple! { "({=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?})", (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10) }
tuple! { "({=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?}, {=?})", (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11) }
