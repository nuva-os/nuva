/*
 * Nuva OS - Kernel - Lib
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.

 */


use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

pub mod ethernet;
pub mod arp;
pub mod ip;
pub mod icmp;
pub mod tcp;
pub mod udp;

/// NetworkInterface
#[derive(Clone, Copy)]
pub struct NetInterface {
 /// InterfaceIndex
 pub ifindex: u32,
 /// MAC Address
 pub mac_addr: [u8; 6],
 /// IPv4 Address
 pub ipv4_addr: u32,
 /// IPv4 ChildnetworkMask
 pub ipv4_mask: u32,
 /// IPv4 networkclose
 pub ipv4_gateway: u32,
 /// MTU
 pub mtu: u32,
 /// Flag
 pub flags: AtomicU32,
 /// ReceiveBytenumber
 pub rx_bytes: AtomicU64,
 /// SendBytenumber
 pub tx_bytes: AtomicU64,
 /// Received packet count
 pub rx_packets: AtomicU64,
 /// Sent packet count
 pub tx_packets: AtomicU64,
}

impl NetInterface {
 /// CreatenewInterface
 pub fn new(ifindex: u32, mac_addr: [u8; 6]) -> Self {
 NetInterface {
 ifindex,
 mac_addr,
 ipv4_addr: 0,
 ipv4_mask: 0,
 ipv4_gateway: 0,
 mtu: 1500,
 flags: AtomicU32::new(0),
 rx_bytes: AtomicU64::new(0),
 tx_bytes: AtomicU64::new(0),
 rx_packets: AtomicU64::new(0),
 tx_packets: AtomicU64::new(0),
 }
 }
 
 /// Set IPv4 Address
 pub fn set_ipv4(&mut self, addr: u32, mask: u32, gateway: u32) {
 self.ipv4_addr = addr;
 self.ipv4_mask = mask;
 self.ipv4_gateway = gateway;
 }
 
 /// ReceiveDataPackage
 pub fn receive(&self, _packet: &[u8]) -> Result<(), NetError> {
 self.rx_bytes.fetch_add(_packet.len() as u64, Ordering::AcqRel);
 self.rx_packets.fetch_add(1, Ordering::AcqRel);
 
 // TODO: HandleDataPackage
 
 Ok(())
 }
 
 /// SendDataPackage
 pub fn transmit(&self, _packet: &[u8]) -> Result<(), NetError> {
 self.tx_bytes.fetch_add(_packet.len() as u64, Ordering::AcqRel);
 self.tx_packets.fetch_add(1, Ordering::AcqRel);
 
 // TODO: SendDataPackage
 
 Ok(())
 }
}

/// Network error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetError {
 /// Success
 Success = 0,
 /// invalidParameter
 InvalidParam = 1,
 /// noneBuffer
 NoBuffer = 2,
 /// Interfacenotexist
 NoInterface = 3,
 /// Addressinvalid
 InvalidAddress = 4,
 /// JoinFailure
 ConnectionFailed = 5,
 /// Timeout
 Timeout = 6,
}

/// NetworkProtocolStack
pub struct NetStack {
 /// Interfacecount
 if_count: AtomicU32,
 /// TCP Joinnumber
 tcp_connections: AtomicU32,
 /// UDP Joinnumber
 udp_connections: AtomicU32,
}

impl NetStack {
 pub const fn new() -> Self {
 NetStack {
 if_count: AtomicU32::new(0),
 tcp_connections: AtomicU32::new(0),
 udp_connections: AtomicU32::new(0),
 }
 }
 
 /// Initialize
 pub fn init(&mut self) {
 // InitializeProtocolSheaf
 ethernet::init_ethernet();
 arp::init_arp();
 ip::init_ip();
 icmp::init_icmp();
 tcp::init_tcp();
 udp::init_udp();
 
 log_info!("Network stack initialized");
 log_info!(" Protocols: Ethernet, ARP, IPv4, ICMP, TCP, UDP");
 }
 
 /// CreateInterface
 pub fn create_interface(&self, mac_addr: [u8; 6]) -> u32 {
 let ifindex = self.if_count.fetch_add(1, Ordering::AcqRel);
 
 log_debug!("Created network interface: ifindex={}, mac={:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
 ifindex, mac_addr[0], mac_addr[1], mac_addr[2], 
 mac_addr[3], mac_addr[4], mac_addr[5]);
 
 ifindex
 }
 
 /// GetInterfacecount
 pub fn get_if_count(&self) -> u32 {
 self.if_count.load(Ordering::Acquire)
 }
 
 /// Get TCP Joinnumber
 pub fn get_tcp_count(&self) -> u32 {
 self.tcp_connections.load(Ordering::Acquire)
 }
 
 /// Get UDP Joinnumber
 pub fn get_udp_count(&self) -> u32 {
 self.udp_connections.load(Ordering::Acquire)
 }
}

/// GlobalNetworkProtocolStack
static mut NET_STACK: NetStack = NetStack::new();

pub fn get_net_stack() -> &'static mut NetStack {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe { &mut NET_STACK }
}

pub fn init_net_stack() {
 let net = get_net_stack();
 net.init();
}