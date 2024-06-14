use std::net::{IpAddr, Ipv4Addr};

use libc::IN_IGNORED;

fn main() {
    let ip_in_hex = 0x0128_A8C0u32;

    // ip_in_hex.swap(0, 3);
    // ip_in_hex.swap(1, 2);

    let reversed = [
        (ip_in_hex) as u8,
        (ip_in_hex >> 8) as u8,
        (ip_in_hex >> 16) as u8,
        (ip_in_hex >> 24) as u8,
    ];

    let ip: Ipv4Addr = reversed.into();
    println!("{}", ip);

    let ip: u32 = u32::from_str_radix("0128A8C0", 16).unwrap();

    // ip_in_hex.swap(0, 3);
    // ip_in_hex.swap(1, 2);

    // let reversed = (ip_in_hex << 24) + (ip_in_hex << 16) + (ip_in_hex << 8) + (ip_in_hex);

    let ip: Ipv4Addr = ip.to_be().into();
    println!("{}", ip);

    let mut ip = 0x0128_A8C0u32;

    // ip_in_hex.swap(0, 3);
    // ip_in_hex.swap(1, 2);

    // let reversed = (ip_in_hex << 24) & (ip_in_hex << 16) & (ip_in_hex << 8) & (ip_in_hex);

    println!("{}", Ipv4Addr::from(ip.to_be()));

    let mut ip_in_hex = 0x0128_A8C0u32.to_le_bytes();

    // ip_in_hex.swap(0, 3);
    // ip_in_hex.swap(1, 2);

    // let reversed = (ip_in_hex << 24) & (ip_in_hex << 16) & (ip_in_hex << 8) & (ip_in_hex);

    println!("{}", Ipv4Addr::from(ip_in_hex));
}
