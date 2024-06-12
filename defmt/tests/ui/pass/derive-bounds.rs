fn main() {
    let baz: Baz<Qux> = Default::default();
    defmt::info!("{}", baz);
}

trait Foo {
    type Bar;
}
#[derive(defmt::Format, Default)]
struct Baz<T: Foo> {
    field: T::Bar,
    field2: Quux<T>,
}
#[derive(defmt::Format, Default)]
struct Qux;
impl Foo for Qux {
    type Bar = Qux;
}
#[allow(dead_code)]
#[derive(defmt::Format, Default)]
enum Quux<T: Foo> {
    #[default]
    None,
    Variant1(T),
    Variant2 {
        f: T::Bar,
    },
    Variant3(T::Bar),
}
