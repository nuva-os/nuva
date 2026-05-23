use crate::{pr_info};
/*
 * Nuva OS - SystemService - Net
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


/// IP Version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpVersion {
    /// IPv4
    V4 = 4,
    /// IPv6
    V6 = 6,
}

/// IP ProtocolType
#[derive(Debug, Clone, Copy)]
pub enum IpProtocol {
    /// ICMP
    Icmp = 1,
    /// TCP
    Tcp = 6,
    /// UDP
    Udp = 17,
}

/// IPv4 Headpart
#[repr(C, packed)]
pub struct Ipv4Header {
    /// VersionsumHeadpartLength
    pub version_ihl: u8,
    /// ServiceType
    pub tos: u8,
    /// totalLength
    pub total_length: u16,
    /// standardidentifier
    pub identification: u16,
    /// FlagsumsliceOffset
    pub flags_fragment: u16,
    /// createexistTime
    pub ttl: u8,
    /// Protocol
    pub protocol: u8,
    /// Headpartchecksum
    pub checksum: u16,
    /// sourceAddress
    pub src_addr: u32,
    /// targetAddress
    pub dst_addr: u32,
}

/// IP Address
#[derive(Debug, Clone, Copy)]
pub struct IpAddress {
    pub version: IpVersion,
    pub addr: [u8; 16],
}

impl IpAddress {
    /// Create IPv4 Address
    pub fn v4(a: u8, b: u8, c: u8, d: u8) -> Self {
        let mut addr = [0u8; 16];
        addr[0] = a;
        addr[1] = b;
        addr[2] = c;
        addr[3] = d;
        IpAddress {
            version: IpVersion::V4,
            addr,
        }
    }
    
    /// convertas u32 (only IPv4)
    pub fn to_u32(&self) -> u32 {
        if self.version == IpVersion::V4 {
            ((self.addr[0] as u32) << 24) |
            ((self.addr[1] as u32) << 16) |
            ((self.addr[2] as u32) << 8) |
            (self.addr[3] as u32)
        } else {
            0
        }
    }
}

/// IP Service
pub struct IpService;

impl IpService {
    pub fn init() -> i32 {
        log_info!("IP service initialized");
        0
    }
    
    /// Send IP Package
    pub fn send_packet(&self, _dst: &IpAddress, _protocol: IpProtocol, _data: &[u8]) -> i32 {
        // TODO: Implementation IP PackageSend
        -1
    }
    
    /// Receive IP Package
    pub fn recv_packet(&self, _buf: &mut [u8]) -> i32 {
        // TODO: Implementation IP PackageReceive
        -1
    }
}

static mut IP_SERVICE: IpService = IpService;

pub fn get_ip_service() -> &'static mut IpService {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &mut IP_SERVICE }
}

pub fn init_ip() {
    IpService::init();
}