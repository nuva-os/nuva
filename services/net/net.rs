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

//! Network kernel module providing MAC, IP, port, buffer, interface, and
//! network manager abstractions.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// A 48-bit MAC (Media Access Control) address.
#[derive(Debug, Clone, Copy, Default)]
pub struct MacAddr(pub [u8; 6]);

impl MacAddr {
    pub const fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    pub fn is_broadcast(&self) -> bool {
        self.0 == [0xFF; 6]
    }

    pub fn is_multicast(&self) -> bool {
        self.0[0] & 0x01 != 0
    }
}

/// A 32-bit IPv4 address.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Ipv4Addr(pub [u8; 4]);

impl Ipv4Addr {
    pub const UNSPECIFIED: Self = Self([0, 0, 0, 0]);
    pub const BROADCAST: Self = Self([255, 255, 255, 255]);
    pub const LOCALHOST: Self = Self([127, 0, 0, 1]);

    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self([a, b, c, d])
    }

    pub fn is_unspecified(&self) -> bool {
        self.0 == [0, 0, 0, 0]
    }

    pub fn is_loopback(&self) -> bool {
        self.0[0] == 127
    }

    pub fn is_private(&self) -> bool {
        match self.0 {
            [10, ..] => true,
            [172, b, ..] if b >= 16 && b <= 31 => true,
            [192, 168, ..] => true,
            _ => false,
        }
    }

    pub fn is_multicast(&self) -> bool {
        self.0[0] >= 224 && self.0[0] <= 239
    }
}

/// A 128-bit IPv6 address.
#[derive(Debug, Clone, Copy, Default)]
pub struct Ipv6Addr(pub [u8; 16]);

impl Ipv6Addr {
    pub const UNSPECIFIED: Self = Self([0; 16]);
    pub const LOCALHOST: Self = Self([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1
    ]);
}

/// A 16-bit network port number with helpers for well-known ranges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Port(pub u16);

impl Port {
    pub const fn new(port: u16) -> Self {
        Self(port)
    }

    pub fn is_system(&self) -> bool {
        self.0 < 1024
    }

    pub fn is_user(&self) -> bool {
        self.0 >= 1024 && self.0 < 49152
    }

    pub fn is_dynamic(&self) -> bool {
        self.0 >= 49152
    }
}

/// Represents a physical or virtual network interface and its current
/// configuration and statistics.
pub struct NetInterface {
    pub id: u32,
    pub name: [u8; 16],
    pub mac: MacAddr,
    pub ipv4: Ipv4Addr,
    pub ipv4_mask: Ipv4Addr,
    pub ipv6: Ipv6Addr,
    pub mtu: u16,
    pub flags: AtomicU32,
    pub rx_bytes: AtomicU64,
    pub tx_bytes: AtomicU64,
    pub rx_packets: AtomicU64,
    pub tx_packets: AtomicU64,
}

impl Clone for NetInterface {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            mac: self.mac.clone(),
            ipv4: self.ipv4.clone(),
            ipv4_mask: self.ipv4_mask.clone(),
            ipv6: self.ipv6.clone(),
            mtu: self.mtu.clone(),
            flags: AtomicU32::new(self.flags.load(core::sync::atomic::Ordering::Relaxed)),
            rx_bytes: AtomicU64::new(self.rx_bytes.load(core::sync::atomic::Ordering::Relaxed)),
            tx_bytes: AtomicU64::new(self.tx_bytes.load(core::sync::atomic::Ordering::Relaxed)),
            rx_packets: AtomicU64::new(self.rx_packets.load(core::sync::atomic::Ordering::Relaxed)),
            tx_packets: AtomicU64::new(self.tx_packets.load(core::sync::atomic::Ordering::Relaxed)),
        }
    }
}

/// Bit flags describing interface capabilities and state.
pub const IFF_UP: u32 = 1 << 0;
pub const IFF_BROADCAST: u32 = 1 << 1;
pub const IFF_LOOPBACK: u32 = 1 << 2;
pub const IFF_MULTICAST: u32 = 1 << 3;
pub const IFF_RUNNING: u32 = 1 << 4;
pub const IFF_PROMISC: u32 = 1 << 5;

/// A fixed-size buffer for sending and receiving network packets.
pub struct NetBuffer {
    pub data: [u8; 2048],
    pub len: usize,
    pub interface_id: u32,
}

impl NetBuffer {
    pub const fn new() -> Self {
        Self {
            data: [0; 2048],
            len: 0,
            interface_id: 0,
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn push(&mut self, bytes: &[u8]) -> bool {
        if self.len + bytes.len() <= 2048 {
            self.data[self.len..self.len + bytes.len()].copy_from_slice(bytes);
            self.len += bytes.len();
            return true;
        }
        false
    }

    pub fn pull(&mut self, len: usize) -> Option<&[u8]> {
        if len <= self.len {
            let slice = &self.data[..len];
            self.len -= len;
            return Some(slice);
        }
        None
    }
}

/// Manages a collection of network interfaces and provides packet I/O.
pub struct NetManager {
    interfaces: [Option<NetInterface>; 16],
    num_interfaces: AtomicU32,
    next_interface_id: AtomicU32,
}

impl NetManager {
    pub const fn new() -> Self {
        Self {
            interfaces: [const { None }; 16],
            num_interfaces: AtomicU32::new(0),
            next_interface_id: AtomicU32::new(1),
        }
    }

    pub fn add_interface(&mut self, _iface: NetInterface) -> u32 { 0 }
    pub fn remove_interface(&mut self, _id: u32) -> bool { false }
    fn get_interface_mut(&mut self, _id: u32) -> Option<&mut NetInterface> { None }
    pub fn send_packet(&mut self, _iface_id: u32, _data: &[u8]) -> bool { false }
    pub fn receive_packet(&mut self, _iface_id: u32) -> Option<&[u8]> { None }
}
