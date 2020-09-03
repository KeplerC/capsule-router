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
mod utils;
mod key;
mod kvs;

use capsule::batch::{Batch, Pipeline, Poll};
use capsule::config::load_config;
use capsule::packets::ip::v4::Ipv4;
use capsule::packets::ip::v6::{Ipv6};
use capsule::packets::ip::ProtocolNumbers;
use capsule::packets::ip::ProtocolNumber;
use capsule::packets::ip::IpPacket;
use capsule::packets::{Ethernet, Packet, Udp};
use capsule::{compose, Mbuf, PortQueue, Runtime};
use chashmap::CHashMap;
use failure::Fallible;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::net::{Ipv4Addr};
use tracing::{debug, Level};
use tracing_subscriber::fmt;

use crate::kvs::*;
use crate::utils::*;
use crate::key::*;

const V4_ADDR: Ipv4Addr = Ipv4Addr::new(203, 0, 113, 1);
static ADDR_MAP: Lazy<CHashMap<Ipv4Addr, Ipv4Addr>> = Lazy::new(CHashMap::new);


// Filter ipv4 packets from ethernet packets
// if it is ipv6 packets, ignore them for now
fn filter_v4(ethernet: Ethernet) -> Fallible<Ipv4>{
    let v4 = ethernet.parse::<Ipv4>()?;
    Ok(v4)
}

#[inline]
fn v4_proc(v4: Ipv4) -> Fallible<Ipv4>{
    Ok(v4)
}


// TODO: finish router selection logic for ipv4
// currently it stays to be constant 
fn router_selection_logic_v4() -> Ipv4Addr {
    V4_ADDR
}


// use chashmap to maintain an ip table 
// look up the key to make connections consistent 
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

// ignore unmatched packet 
fn ignore_packet(ethernet: Ethernet) -> Fallible<Ethernet>{
    println!("Umatched");
    Ok(ethernet)
}


fn proc_v6_udp(ethernet: Ethernet) -> Fallible<Ethernet> {

    let v6 = ethernet.peek::<Ipv6>()?;
    dump_v6(&v6);
    let udp = v6.peek::<Udp<Ipv6>>()?;
    dump_udp(&udp);

    let mut reply = Mbuf::new()?;
    let mut reply = reply.push::<Ethernet>()?;
    reply.set_src(ethernet.dst());
    reply.set_dst(ethernet.src());
    let mut reply = reply.push::<Ipv6>()?;
    reply.set_src(v6.src());
    reply.set_dst(v6.src());
    reply.reconcile_all();
    dump_v6(&reply);

    let mut reply_ethernet = reply.reset().parse::<Ethernet>()?;
    Ok(reply_ethernet)
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
            //classify current protocol as 
            // * ProtocolNumbers::Tcp 
            // * ProtocolNumbers::Udp
            // * others 
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
    .send(qs["eth2"].clone()) //forward to ethernet port 2, it can be eth1 
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
