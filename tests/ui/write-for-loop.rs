struct S;

impl defmt::Format for S {
    fn format(&self, f: defmt::Formatter) {
        for _ in 0..3 {
            defmt::write!(f, "hello");
        }
    }
}

fn main() {}
