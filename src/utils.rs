use capsule::batch::{Batch, Either, Pipeline, Poll};
use capsule::config::load_config;
use capsule::packets::ip::v4::Ipv4;
use capsule::packets::ip::v6::{Ipv6, Ipv6Packet};
use capsule::packets::ip::ProtocolNumbers;
use capsule::packets::ip::ProtocolNumber;
use capsule::packets::ip::IpPacket;
use capsule::packets::{EtherTypes, Ethernet, Packet, Tcp, Udp};
use capsule::{compose, Mbuf, PortQueue, Runtime};
use chashmap::CHashMap;
use failure::Fallible;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::atomic::{AtomicU16, Ordering};
use tracing::{debug, Level};
use colored::*;
use tracing_subscriber::fmt;

use std::sync::Arc;

use std::net::TcpStream;
use std::io::{Read, Write, stdout};
use std::io;
use std::io::prelude::*;
use std::fs::File;

use rustls;
use webpki;
use webpki_roots;



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