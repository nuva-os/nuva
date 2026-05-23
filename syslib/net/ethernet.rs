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


/// EthernetFrameHead
#[repr(C, packed)]
pub struct EthernetHeader {
    /// target MAC
    pub dst_mac: [u8; 6],
    /// source MAC
    pub src_mac: [u8; 6],
    /// EthernetType
    pub ethertype: u16,
}

/// EthernetType
pub mod ethertype {
    pub const IPV4: u16 = 0x0800;
    pub const ARP: u16 = 0x0806;
    pub const IPV6: u16 = 0x86DD;
    pub const VLAN: u16 = 0x8100;
}

/// EthernetFrame
pub struct EthernetFrame {
    pub header: EthernetHeader,
    pub payload: &'static [u8],
}

impl EthernetFrame {
    /// parseEthernetFrame
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 14 {
            return None;
        }
        
        let header = EthernetHeader {
            dst_mac: [data[0], data[1], data[2], data[3], data[4], data[5]],
            src_mac: [data[6], data[7], data[8], data[9], data[10], data[11]],
            ethertype: ((data[12] as u16) << 8) | (data[13] as u16),
        };
        
        Some(EthernetFrame {
            header,
            // SAFETY: unsafe block required for low-level memory or hardware access
            payload: unsafe { core::mem::transmute(&data[14..]) },
        })
    }
    
    /// BuildEthernetFrame
    pub fn build(&self, buf: &mut [u8]) -> usize {
        if buf.len() < 14 + self.payload.len() {
            return 0;
        }
        
        buf[0..6].copy_from_slice(&self.header.dst_mac);
        buf[6..12].copy_from_slice(&self.header.src_mac);
        buf[12] = (self.header.ethertype >> 8) as u8;
        buf[13] = self.header.ethertype as u8;
        buf[14..14+self.payload.len()].copy_from_slice(self.payload);
        
        14 + self.payload.len()
    }
}

/// InitializeEthernetSheaf
pub fn init_ethernet() {
    log_info!("Ethernet layer initialized");
}