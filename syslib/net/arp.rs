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


use core::sync::atomic::{AtomicU32, Ordering};

/// ARP Head
#[repr(C, packed)]
pub struct ArpHeader {
 /// hardcaseType
 pub htype: u16,
 /// ProtocolType
 pub ptype: u16,
 /// hardcaseAddressLength
 pub hlen: u8,
 /// ProtocolAddressLength
 pub plen: u8,
 /// Operationcode
 pub oper: u16,
 /// Sendmethod MAC
 pub sha: [u8; 6],
 /// Sendmethod IP
 pub spa: u32,
 /// targetmethod MAC
 pub tha: [u8; 6],
 /// targetmethod IP
 pub tpa: u32,
}

/// ARP Operationcode
pub mod arp_oper {
 pub const REQUEST: u16 = 1;
 pub const REPLY: u16 = 2;
}

/// ARP formproject
pub struct ArpEntry {
 /// IP Address
 pub ip_addr: u32,
 /// MAC Address
 pub mac_addr: [u8; 6],
 /// Flag
 pub flags: AtomicU32,
}

/// ARP form
pub struct ArpTable {
 /// formprojectcount
 count: AtomicU32,
}

impl ArpTable {
 pub const fn new() -> Self {
 ArpTable {
 count: AtomicU32::new(0),
 }
 }
 
 /// Find MAC Address
 pub fn lookup(&self, _ip_addr: u32) -> Option<[u8; 6]> {
 // TODO: Implementation ARP formFind
 None
 }
 
 /// addPlusformproject
 pub fn add(&self, _ip_addr: u32, _mac_addr: [u8; 6]) {
 self.count.fetch_add(1, Ordering::AcqRel);
 }
 
 /// Getformprojectcount
 pub fn get_count(&self) -> u32 {
 self.count.load(Ordering::Acquire)
 }
}

/// Global ARP form
static mut ARP_TABLE: ArpTable = ArpTable::new();

pub fn get_arp_table() -> &'static mut ArpTable {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe { &mut ARP_TABLE }
}

/// Initialize ARP
pub fn init_arp() {
 log_info!("ARP protocol initialized");
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_arp_oper() {
 assert_eq!(arp_oper::REQUEST, 1);
 assert_eq!(arp_oper::REPLY, 2);
 }

 #[test]
 fn test_arp_header_size() {
 // ARP Headpartsolidfixed 28 Byte
 assert_eq!(core::mem::size_of::<ArpHeader>(), 28);
 }

 #[test]
 fn test_arp_header_fields() {
 let header = ArpHeader {
 htype: 1, // Ethernet
 ptype: 0x0800, // IPv4
 hlen: 6,
 plen: 4,
 oper: arp_oper::REQUEST,
 sha: [0x00, 0x11, 0x22, 0x33, 0x44, 0x55],
 spa: 0xC0A80001, // 192.168.0.1
 tha: [0; 6],
 tpa: 0xC0A80002, // 192.168.0.2
 };

 assert_eq!(header.htype, 1);
 assert_eq!(header.ptype, 0x0800);
 assert_eq!(header.hlen, 6);
 assert_eq!(header.plen, 4);
 assert_eq!(header.oper, arp_oper::REQUEST);
 }

 #[test]
 fn test_arp_header_request() {
 let header = ArpHeader {
 htype: 1,
 ptype: 0x0800,
 hlen: 6,
 plen: 4,
 oper: arp_oper::REQUEST,
 sha: [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF],
 spa: 0xC0A80001,
 tha: [0; 6], // Requesttimetarget MAC 
 tpa: 0xC0A80002,
 };

 assert_eq!(header.oper, 1);
 assert_eq!(header.tha, [0; 6]);
 }

 #[test]
 fn test_arp_header_reply() {
 let header = ArpHeader {
 htype: 1,
 ptype: 0x0800,
 hlen: 6,
 plen: 4,
 oper: arp_oper::REPLY,
 sha: [0x11, 0x22, 0x33, 0x44, 0x55, 0x66],
 spa: 0xC0A80002,
 tha: [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF],
 tpa: 0xC0A80001,
 };

 assert_eq!(header.oper, 2);
 }

 #[test]
 fn test_arp_entry() {
 let entry = ArpEntry {
 ip_addr: 0xC0A80001,
 mac_addr: [0x00, 0x11, 0x22, 0x33, 0x44, 0x55],
 flags: AtomicU32::new(1),
 };

 assert_eq!(entry.ip_addr, 0xC0A80001);
 assert_eq!(entry.mac_addr, [0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
 }

 #[test]
 fn test_arp_table_new() {
 let table = ArpTable::new();

 assert_eq!(table.get_count(), 0);
 }

 #[test]
 fn test_arp_table_add() {
 let table = ArpTable::new();

 table.add(0xC0A80001, [0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);

 assert_eq!(table.get_count(), 1);
 }

 #[test]
 fn test_arp_table_multiple() {
 let table = ArpTable::new();

 table.add(0xC0A80001, [0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
 table.add(0xC0A80002, [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
 table.add(0xC0A80003, [0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);

 assert_eq!(table.get_count(), 3);
 }

 #[test]
 fn test_arp_table_lookup_empty() {
 let table = ArpTable::new();

 let result = table.lookup(0xC0A80001);
 assert!(result.is_none());
 }
}