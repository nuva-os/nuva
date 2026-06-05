/*
 * Nuva OS - Kernel - Net - Ethernet
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
/*
 * Nuva OS - Kernel - Ethernet Protocol
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Ethernet (IEEE 802.3) protocol implementation.
 */

/// Ethernet Header
#[repr(C, packed)]
pub struct EthernetHeader {
    /// Destination MAC address
    pub h_dest: [u8; 6],
    /// Source MAC address
    pub h_source: [u8; 6],
    /// Protocol type
    pub h_proto: u16,
}

impl EthernetHeader {
    /// Header size
    pub const SIZE: usize = 14;
    
    /// Create new header
    pub fn new(dest: &[u8; 6], source: &[u8; 6], proto: u16) -> Self {
        let mut h_dest = [0u8; 6];
        let mut h_source = [0u8; 6];
        h_dest.copy_from_slice(dest);
        h_source.copy_from_slice(source);
        
        EthernetHeader {
            h_dest,
            h_source,
            h_proto: proto.to_be(),
        }
    }
    
    /// Get protocol type (host byte order)
    pub fn get_proto(&self) -> u16 {
        u16::from_be(self.h_proto)
    }
}

/// Ethernet Protocol Types
pub mod eth_type {
    /// IP protocol
    pub const IP: u16 = 0x0800;
    /// ARP protocol
    pub const ARP: u16 = 0x0806;
    /// RARP protocol
    pub const RARP: u16 = 0x8035;
    /// VLAN tagged
    pub const VLAN: u16 = 0x8100;
    /// IPv6
    pub const IPV6: u16 = 0x86DD;
    /// PPPoE discovery
    pub const PPPOE_DISC: u16 = 0x8863;
    /// PPPoE session
    pub const PPPOE_SESS: u16 = 0x8864;
    /// Link Layer Discovery Protocol
    pub const LLDP: u16 = 0x88CC;
    /// Wake-on-LAN
    pub const WOL: u16 = 0x0842;
    /// Loopback
    pub const LOOP: u16 = 0x0060;
    /// 802.1Q QinQ
    pub const QinQ: u16 = 0x88A8;
    /// EAPOL
    pub const EAPOL: u16 = 0x888E;
    /// PAE
    pub const PAE: u16 = 0x888E;
    /// AOE
    pub const AOE: u16 = 0x88A2;
    /// TRILL
    pub const TRILL: u16 = 0x22EB;
    /// FCOE
    pub const FCOE: u16 = 0x8906;
    /// FIP
    pub const FIP: u16 = 0x8914;
    /// MPLS unicast
    pub const MPLS_UC: u16 = 0x8847;
    /// MPLS multicast
    pub const MPLS_MC: u16 = 0x8848;
    /// TEB (Transparent Ethernet Bridging)
    pub const TEB: u16 = 0x6558;
    /// 802.3 Slow protocols
    pub const SLOW: u16 = 0x8809;
    /// 802.1ad Service VLAN
    pub const S_VLAN: u16 = 0x88A8;
}

/// Ethernet Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EthernetType {
    /// IP
    Ip = 0x0800,
    /// ARP
    Arp = 0x0806,
    /// VLAN
    Vlan = 0x8100,
    /// IPv6
    Ipv6 = 0x86DD,
    /// PPPoE Discovery
    PppoeDisc = 0x8863,
    /// PPPoE Session
    PppoeSess = 0x8864,
    /// LLDP
    Lldp = 0x88CC,
    /// Unknown
    Unknown = 0,
}

impl From<u16> for EthernetType {
    fn from(value: u16) -> Self {
        match value {
            0x0800 => EthernetType::Ip,
            0x0806 => EthernetType::Arp,
            0x8100 => EthernetType::Vlan,
            0x86DD => EthernetType::Ipv6,
            0x8863 => EthernetType::PppoeDisc,
            0x8864 => EthernetType::PppoeSess,
            0x88CC => EthernetType::Lldp,
            _ => EthernetType::Unknown,
        }
    }
}

/// VLAN Header (802.1Q)
#[repr(C, packed)]
pub struct VlanHeader {
    /// VLAN TCI (Tag Control Information)
    pub tci: u16,
    /// Encapsulated protocol
    pub encap_proto: u16,
}

impl VlanHeader {
    /// Header size
    pub const SIZE: usize = 4;
    
    /// Get VLAN ID
    pub fn get_vid(&self) -> u16 {
        u16::from_be(self.tci) & 0x0FFF
    }
    
    /// Get priority (PCP)
    pub fn get_pcp(&self) -> u8 {
        ((u16::from_be(self.tci) >> 13) & 0x7) as u8
    }
    
    /// Get CFI (Canonical Format Indicator)
    pub fn get_cfi(&self) -> u8 {
        ((u16::from_be(self.tci) >> 12) & 0x1) as u8
    }
}

/// Ethernet Utilities
pub struct EthernetUtils;

impl EthernetUtils {
    /// Check if address is broadcast
    pub fn is_broadcast(addr: &[u8; 6]) -> bool {
        addr == &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
    }
    
    /// Check if address is multicast
    pub fn is_multicast(addr: &[u8; 6]) -> bool {
        (addr[0] & 0x01) != 0
    }
    
    /// Check if address is unicast
    pub fn is_unicast(addr: &[u8; 6]) -> bool {
        !Self::is_multicast(addr) && !Self::is_broadcast(addr)
    }
    
    /// Check if address is locally administered
    pub fn is_local(addr: &[u8; 6]) -> bool {
        (addr[0] & 0x02) != 0
    }
    
    /// Check if address is zero
    pub fn is_zero(addr: &[u8; 6]) -> bool {
        addr == &[0, 0, 0, 0, 0, 0]
    }
    
    /// Generate random MAC address
    pub fn random_mac() -> [u8; 6] {
        // TODO: Use proper random number generator
        [0x02, 0x00, 0x00, 0x00, 0x00, 0x01]
    }
    
    /// Convert MAC address to string
    pub fn mac_to_string(addr: &[u8; 6], buf: &mut [u8]) -> usize {
        let hex = b"0123456789abcdef";
        let mut pos = 0;
        
        for (i, byte) in addr.iter().enumerate() {
            if i > 0 {
                buf[pos] = b':';
                pos += 1;
            }
            buf[pos] = hex[(byte >> 4) as usize];
            buf[pos + 1] = hex[(byte & 0x0F) as usize];
            pos += 2;
        }
        
        pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ethernet_header() {
        let dest = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
        let src = [0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB];
        let hdr = EthernetHeader::new(&dest, &src, eth_type::IP);
        
        assert_eq!(hdr.h_dest, dest);
        assert_eq!(hdr.h_source, src);
        assert_eq!(hdr.get_proto(), eth_type::IP);
    }
    
    #[test]
    fn test_ethernet_utils() {
        let broadcast = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let multicast = [0x01, 0x00, 0x5E, 0x00, 0x00, 0x01];
        let unicast = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
        
        assert!(EthernetUtils::is_broadcast(&broadcast));
        assert!(EthernetUtils::is_multicast(&multicast));
        assert!(EthernetUtils::is_unicast(&unicast));
    }
}
