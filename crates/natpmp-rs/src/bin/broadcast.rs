use std::net::Ipv4Addr;

fn main() {
    let ip: u32 = u32::from_str_radix("0128A8C0", 16).unwrap();

    let reversed = ip.swap_bytes();

    // // ip_in_hex.swap(0, 3);
    // // ip_in_hex.swap(1, 2);

    // let reversed = [
    //     (ip_in_hex) as u8,
    //     (ip_in_hex >> 8) as u8,
    //     (ip_in_hex >> 16) as u8,
    //     (ip_in_hex >> 24) as u8,
    // ];

    let ip: Ipv4Addr = reversed.into();
    println!("{}", ip);
}
