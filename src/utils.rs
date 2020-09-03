

use capsule::packets::ip::v4::Ipv4;
use capsule::packets::ip::v6::{Ipv6};
use capsule::packets::ip::IpPacket;
use capsule::packets::{Ethernet, Packet, Tcp, Udp};
use capsule::{ Mbuf};
use failure::Fallible;
use tracing::{debug, Level};
use colored::*;



// dump basic ethernet packet infomation (headers)

pub fn dump_eth(packet: Mbuf) -> Fallible<Ethernet> {
    let ethernet = packet.parse::<Ethernet>()?;

    let info_fmt = format!("{:?}", ethernet).magenta().bold();
    println!("{}", info_fmt);

    Ok(ethernet)
}

// dump ipv4 packet information (headers)

pub fn dump_v4(v4: &Ipv4){
    let info_fmt = format!("{:?}", v4).yellow();
    println!("{}", info_fmt);
}

// dump ipv6 packet information (headers)

pub fn dump_v6(v6: &Ipv6){
    let info_fmt = format!("{:?}", v6).cyan();
    println!("{}", info_fmt);
}

// dump tcp packet and tcp flow information 
pub fn dump_tcp<T: IpPacket>(tcp: &Tcp<T>) {
    let tcp_fmt = format!("{:?}", tcp).green();
    println!("{}", tcp_fmt);

    let flow_fmt = format!("{:?}", tcp.flow()).bright_blue();
    println!("{}", flow_fmt);
}

// dump udp packet information(headers)
pub fn dump_udp<T: IpPacket>(udp: &Udp<T>) {
    let udp_fmt = format!("{:?}", udp).green();
    println!("{}", udp_fmt);

}