fn main() {
    defmt::info!("hello");
}

#[defmt::global_logger]
struct Logger;

unsafe impl defmt::Logger for Logger {
    fn acquire() -> Option<core::ptr::NonNull<dyn defmt::Write>> {
        None
    }
    unsafe fn release(_writer: core::ptr::NonNull<dyn defmt::Write>) {}
}

defmt::timestamp!("{=u32}", 0);
