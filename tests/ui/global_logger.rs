#![no_main]
#![no_std]

#[defmt::global_logger]
use defmt::Logger;

struct Logger;

impl defmt::Write for Logger {
  fn write(&mut self, _: &[u8]) {
    write!(42, "hello");
  }
}
