use stdout_logger as _;

fn main() {
    println!("Hello, world!");
    defmt::error!("This should be 5: {}", 5);
}
