use super::*;

macro_rules! write_to_le_bytes {
    ($s:ident) => {
        /// Implementation detail
        pub fn $s(b: &$s) {
            write(&b.to_le_bytes())
        }
    };
}

write_to_le_bytes!(u8);
write_to_le_bytes!(u16);
write_to_le_bytes!(u32);
write_to_le_bytes!(u64);
write_to_le_bytes!(u128);
write_to_le_bytes!(usize);
write_to_le_bytes!(i8);
write_to_le_bytes!(i16);
write_to_le_bytes!(i32);
write_to_le_bytes!(i64);
write_to_le_bytes!(i128);
write_to_le_bytes!(isize);
