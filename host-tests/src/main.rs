mod stdout;

fn main() {
    let value = host_tests::add(1, 2);
    defmt::error!("this is error level output - value = {=u64}", value);
}

use std::time::SystemTime;

defmt::timestamp!("{=u128:tus}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_micros());
