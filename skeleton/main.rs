/*
* Copyright 2019 Comcast Cable Communications Management, LLC
*
* Licensed under the Apache License, Version 2.0 (the "License");
* you may not use this file except in compliance with the License.
* You may obtain a copy of the License at
*
* http://www.apache.org/licenses/LICENSE-2.0
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License.
*
* SPDX-License-Identifier: Apache-2.0
*/

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

use rustls::Session;

const V4_ADDR: Ipv4Addr = Ipv4Addr::new(203, 0, 113, 1);
const selection_logic:i32 = 0; //0 = constant, 1 = round robin, 2 = random  
static ADDR_MAP: Lazy<CHashMap<Ipv4Addr, Ipv4Addr>> = Lazy::new(CHashMap::new);

#[inline]
fn dump_eth(packet: Mbuf) -> Fallible<Ethernet> {
    let ethernet = packet.parse::<Ethernet>()?;

    let info_fmt = format!("{:?}", ethernet).magenta().bold();
    println!("{}", info_fmt);

    Ok(ethernet)
}

#[inline]
fn dump_v4(v4: &Ipv4){
    let info_fmt = format!("{:?}", v4).yellow();
    println!("{}", info_fmt);
}

#[inline]
fn dump_v6(v6: &Ipv6){
    let info_fmt = format!("{:?}", v6).cyan();
    println!("{}", info_fmt);
}

#[inline]
fn dump_tcp<T: IpPacket>(tcp: &Tcp<T>) {
    let tcp_fmt = format!("{:?}", tcp).green();
    println!("{}", tcp_fmt);

    let flow_fmt = format!("{:?}", tcp.flow()).bright_blue();
    println!("{}", flow_fmt);
}


#[inline]
fn dump_udp<T: IpPacket>(udp: &Udp<T>) {
    let udp_fmt = format!("{:?}", udp).green();
    println!("{}", udp_fmt);

}

fn filter_v4(ethernet: Ethernet) -> Fallible<Ipv4>{
    let v4 = ethernet.parse::<Ipv4>()?;
    Ok(v4)
}

#[inline]
fn v4_proc(v4: Ipv4) -> Fallible<Ipv4>{
    println!("haha");
    Ok(v4)
}

fn router_selection_logic_v4() -> Ipv4Addr {
    V4_ADDR
}


fn figure_out_dst_v4(src: Ipv4Addr) -> Ipv4Addr{
    let key = src;
    if let Some(value) = ADDR_MAP.get(&key) {
        *value
    } else {
        let assigned_addr = router_selection_logic_v4();
        ADDR_MAP.insert_new(key, assigned_addr);
        assigned_addr
    }
}

fn ignore_packet(ethernet: Ethernet) -> Fallible<Ethernet>{
    println!("Umatched");
    Ok(ethernet)
}


fn proc_v4_udp(ethernet: Ethernet) -> Fallible<Ethernet> {

    let v4 = ethernet.peek::<Ipv4>()?;
    dump_v4(&v4);
    let udp = v4.peek::<Udp<Ipv4>>()?;
    dump_udp(&udp);

    let mut reply = Mbuf::new()?;
    let mut reply = reply.push::<Ethernet>()?;
    reply.set_src(ethernet.dst());
    reply.set_dst(ethernet.src());
    let mut reply = reply.push::<Ipv4>()?;
    reply.set_src(v4.src());
    reply.set_dst(figure_out_dst_v4(v4.src()));
    reply.set_ttl(255);
    reply.reconcile_all();
    dump_v4(&reply);

    let mut reply_ethernet = reply.reset().parse::<Ethernet>()?;
    Ok(reply_ethernet)
}

fn get_protocol(ethernet: &Ethernet) -> ProtocolNumber{
    let v4 = ethernet.peek::<Ipv4>().unwrap();
    v4.protocol()
}

fn install(qs: HashMap<String, PortQueue>) -> impl Pipeline {
    Poll::new(qs["eth1"].clone())
    .map(dump_eth)
    .group_by(
        |packet| {
            get_protocol(packet)
        },
        |groups| {
            compose!( groups {
                ProtocolNumbers::Tcp => |group| {
                    group.map(|pkt| {
                        ignore_packet(pkt)
                    })
                }
                ProtocolNumbers::Udp => |group| {
                    group.map(|pkt| {
                        proc_v4_udp(pkt)
                    })
                }
                _ => |group| {
                    group.map(|pkt| {
                        ignore_packet(pkt)
                    })
                }
            })
        },
    )
    .send(qs["eth2"].clone())
}


fn main() -> Fallible<()> {
    let subscriber = fmt::Subscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cfg = load_config()?;
    debug!(?cfg);

    Runtime::build(cfg)?.add_pipeline_to_core(0, install)?.execute()
}
