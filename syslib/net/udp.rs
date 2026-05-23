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

/// UDP Head
#[repr(C, packed)]
pub struct UdpHeader {
 /// sourcePort
 pub src_port: u16,
 /// targetPort
 pub dst_port: u16,
 /// Length
 pub length: u16,
 /// Verifysum
 pub checksum: u16,
}

/// UDP Join
pub struct UdpConnection {
 /// LocalPort
 pub local_port: u16,
 /// farprocessPort
 pub remote_port: u16,
 /// Local IP
 pub local_ip: u32,
 /// farprocess IP
 pub remote_ip: u32,
}

impl UdpConnection {
 /// CreatenewJoin
 pub fn new(local_port: u16, local_ip: u32) -> Self {
 UdpConnection {
 local_port,
 remote_port: 0,
 local_ip,
 remote_ip: 0,
 }
 }
}

/// UDP statistics
pub struct UdpStats {
 /// SendDatanumber
 pub datagrams_tx: AtomicU64,
 /// ReceiveDatanumber
 pub datagrams_rx: AtomicU64,
 /// SendBytenumber
 pub bytes_tx: AtomicU64,
 /// ReceiveBytenumber
 pub bytes_rx: AtomicU64,
 /// Errornumber
 pub errors: AtomicU32,
}

impl UdpStats {
 pub const fn new() -> Self {
 UdpStats {
 datagrams_tx: AtomicU64::new(0),
 datagrams_rx: AtomicU64::new(0),
 bytes_tx: AtomicU64::new(0),
 bytes_rx: AtomicU64::new(0),
 errors: AtomicU32::new(0),
 }
 }
}

/// Global UDP Statistics
static mut UDP_STATS: UdpStats = UdpStats::new();

pub fn get_udp_stats() -> &'static UdpStats {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe { &UDP_STATS }
}

/// Initialize UDP
pub fn init_udp() {
 log_info!("UDP protocol initialized");
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_udp_header_size() {
 // UDP Headpartsolidfixed 8 Byte
 assert_eq!(core::mem::size_of::<UdpHeader>(), 8);
 }

 #[test]
 fn test_udp_connection_new() {
 let conn = UdpConnection::new(53, 0xC0A80001);
 assert_eq!(conn.local_port, 53);
 assert_eq!(conn.local_ip, 0xC0A80001);
 assert_eq!(conn.remote_port, 0);
 assert_eq!(conn.remote_ip, 0);
 }

 #[test]
 fn test_udp_stats_new() {
 let stats = UdpStats::new();
 assert_eq!(stats.datagrams_tx.load(Ordering::Relaxed), 0);
 assert_eq!(stats.datagrams_rx.load(Ordering::Relaxed), 0);
 assert_eq!(stats.bytes_tx.load(Ordering::Relaxed), 0);
 assert_eq!(stats.bytes_rx.load(Ordering::Relaxed), 0);
 assert_eq!(stats.errors.load(Ordering::Relaxed), 0);
 }

 #[test]
 fn test_udp_stats_increment() {
 let stats = UdpStats::new();

 stats.datagrams_tx.fetch_add(10, Ordering::Relaxed);
 stats.datagrams_rx.fetch_add(5, Ordering::Relaxed);
 stats.bytes_tx.fetch_add(1024, Ordering::Relaxed);
 stats.bytes_rx.fetch_add(512, Ordering::Relaxed);
 stats.errors.fetch_add(1, Ordering::Relaxed);

 assert_eq!(stats.datagrams_tx.load(Ordering::Relaxed), 10);
 assert_eq!(stats.datagrams_rx.load(Ordering::Relaxed), 5);
 assert_eq!(stats.bytes_tx.load(Ordering::Relaxed), 1024);
 assert_eq!(stats.bytes_rx.load(Ordering::Relaxed), 512);
 assert_eq!(stats.errors.load(Ordering::Relaxed), 1);
 }

 #[test]
 fn test_udp_connection_fields() {
 let mut conn = UdpConnection::new(8080, 0x0A000001);

 // SetRemote address
 conn.remote_port = 9000;
 conn.remote_ip = 0x0A000002;

 assert_eq!(conn.local_port, 8080);
 assert_eq!(conn.local_ip, 0x0A000001);
 assert_eq!(conn.remote_port, 9000);
 assert_eq!(conn.remote_ip, 0x0A000002);
 }

 #[test]
 fn test_udp_header_fields() {
 let header = UdpHeader {
 src_port: 12345,
 dst_port: 80,
 length: 100,
 checksum: 0xABCD,
 };

 assert_eq!(header.src_port, 12345);
 assert_eq!(header.dst_port, 80);
 assert_eq!(header.length, 100);
 assert_eq!(header.checksum, 0xABCD);
 }
}