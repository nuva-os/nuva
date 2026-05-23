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


use core::sync::atomic::{AtomicU64, Ordering};

/// ICMP Head
#[repr(C, packed)]
pub struct IcmpHeader {
 /// Type
 pub icmp_type: u8,
 /// Code
 pub code: u8,
 /// Verifysum
 pub checksum: u16,
 /// itsremainderpartsplit (takedecideType)
 pub rest: u32,
}

/// ICMP Type
pub mod icmp_type {
 pub const ECHO_REPLY: u8 = 0;
 pub const DEST_UNREACH: u8 = 3;
 pub const SOURCE_QUENCH: u8 = 4;
 pub const REDIRECT: u8 = 5;
 pub const ECHO_REQUEST: u8 = 8;
 pub const TIME_EXCEEDED: u8 = 11;
 pub const PARAM_PROBLEM: u8 = 12;
 pub const TIMESTAMP: u8 = 13;
 pub const TIMESTAMP_REPLY: u8 = 14;
}

/// ICMP statistics
pub struct IcmpStats {
 /// Send Echo Request
 pub echo_req_tx: AtomicU64,
 /// Receive Echo Reply
 pub echo_rep_rx: AtomicU64,
 /// Receive Echo Request
 pub echo_req_rx: AtomicU64,
 /// Send Echo Reply
 pub echo_rep_tx: AtomicU64,
}

impl IcmpStats {
 pub const fn new() -> Self {
 IcmpStats {
 echo_req_tx: AtomicU64::new(0),
 echo_rep_rx: AtomicU64::new(0),
 echo_req_rx: AtomicU64::new(0),
 echo_rep_tx: AtomicU64::new(0),
 }
 }
}

/// Global ICMP Statistics
static mut ICMP_STATS: IcmpStats = IcmpStats::new();

pub fn get_icmp_stats() -> &'static IcmpStats {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe { &ICMP_STATS }
}

/// Initialize ICMP
pub fn init_icmp() {
 log_info!("ICMP protocol initialized");
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_icmp_type_constants() {
 assert_eq!(icmp_type::ECHO_REPLY, 0);
 assert_eq!(icmp_type::DEST_UNREACH, 3);
 assert_eq!(icmp_type::SOURCE_QUENCH, 4);
 assert_eq!(icmp_type::REDIRECT, 5);
 assert_eq!(icmp_type::ECHO_REQUEST, 8);
 assert_eq!(icmp_type::TIME_EXCEEDED, 11);
 assert_eq!(icmp_type::PARAM_PROBLEM, 12);
 assert_eq!(icmp_type::TIMESTAMP, 13);
 assert_eq!(icmp_type::TIMESTAMP_REPLY, 14);
 }

 #[test]
 fn test_icmp_header_size() {
 // ICMP Headpartsolidfixed 8 Byte
 assert_eq!(core::mem::size_of::<IcmpHeader>(), 8);
 }

 #[test]
 fn test_icmp_header_fields() {
 let header = IcmpHeader {
 icmp_type: icmp_type::ECHO_REQUEST,
 code: 0,
 checksum: 0x1234,
 rest: 0x00010001, // id=1, seq=1
 };

 assert_eq!(header.icmp_type, icmp_type::ECHO_REQUEST);
 assert_eq!(header.code, 0);
 assert_eq!(header.checksum, 0x1234);
 }

 #[test]
 fn test_icmp_stats_new() {
 let stats = IcmpStats::new();

 assert_eq!(stats.echo_req_tx.load(Ordering::Relaxed), 0);
 assert_eq!(stats.echo_rep_rx.load(Ordering::Relaxed), 0);
 assert_eq!(stats.echo_req_rx.load(Ordering::Relaxed), 0);
 assert_eq!(stats.echo_rep_tx.load(Ordering::Relaxed), 0);
 }

 #[test]
 fn test_icmp_stats_increment() {
 let stats = IcmpStats::new();

 stats.echo_req_tx.fetch_add(1, Ordering::Relaxed);
 stats.echo_rep_rx.fetch_add(1, Ordering::Relaxed);

 assert_eq!(stats.echo_req_tx.load(Ordering::Relaxed), 1);
 assert_eq!(stats.echo_rep_rx.load(Ordering::Relaxed), 1);
 }

 #[test]
 fn test_icmp_echo_request() {
 let header = IcmpHeader {
 icmp_type: icmp_type::ECHO_REQUEST,
 code: 0,
 checksum: 0,
 rest: 0x00010001,
 };

 assert_eq!(header.icmp_type, 8);
 }

 #[test]
 fn test_icmp_echo_reply() {
 let header = IcmpHeader {
 icmp_type: icmp_type::ECHO_REPLY,
 code: 0,
 checksum: 0,
 rest: 0x00010001,
 };

 assert_eq!(header.icmp_type, 0);
 }

 #[test]
 fn test_icmp_dest_unreach() {
 let header = IcmpHeader {
 icmp_type: icmp_type::DEST_UNREACH,
 code: 1, // Host unreachable
 checksum: 0,
 rest: 0,
 };

 assert_eq!(header.icmp_type, 3);
 assert_eq!(header.code, 1);
 }

 #[test]
 fn test_icmp_time_exceeded() {
 let header = IcmpHeader {
 icmp_type: icmp_type::TIME_EXCEEDED,
 code: 0, // TTL exceeded
 checksum: 0,
 rest: 0,
 };

 assert_eq!(header.icmp_type, 11);
 }
}