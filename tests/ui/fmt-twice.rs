struct S;

impl defmt::Format for S {
    fn format(&self, f: defmt::Formatter) {
        0u8.format(f);
        0u16.format(f);
    }
}

fn main() {}
