#![no_std]
#![no_main]

use core::net::{
    AddrParseError, IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6,
};

use cortex_m_rt::entry;
use cortex_m_semihosting::debug;

use defmt_semihosting as _; // global logger

#[entry]
fn main() -> ! {
    let a = Ipv4Addr::new(127, 0, 0, 1);
    let b = IpAddr::V4(a);
    let c = SocketAddrV4::new(a, 8080);
    let d = SocketAddr::V4(c);

    let e = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1);
    let f = IpAddr::V6(e);
    let g = SocketAddrV6::new(e, 8080, 0, 0);
    let h = SocketAddr::V6(g);

    let i: AddrParseError = "127.0.0.1:8080".parse::<IpAddr>().unwrap_err();

    defmt::dbg!(a, b, c, d, e, f, g, h, i);

    loop {
        debug::exit(debug::EXIT_SUCCESS)
    }
}

// like `panic-semihosting` but doesn't print to stdout (that would corrupt the defmt stream)
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        debug::exit(debug::EXIT_FAILURE)
    }
}
