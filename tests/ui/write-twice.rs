struct S;

impl defmt::Format for S {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "hello");
        defmt::write!(f, "world");
    }
}

fn main() {}
