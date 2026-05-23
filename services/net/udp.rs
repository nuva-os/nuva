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

//! UDP ProtocolImplementation

use core::sync::atomic::{AtomicU16, AtomicU32, AtomicU64, Ordering};

/// UDP Headpart
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct UdpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub length: u16,
    pub checksum: u16,
}

impl UdpHeader {
    pub const fn new() -> Self {
        Self {
            src_port: 0,
            dst_port: 0,
            length: 8,
            checksum: 0,
        }
    }

    pub fn payload_len(&self) -> usize {
        self.length.saturating_sub(8) as usize
    }
}

/// UDP suiteacceptWord
pub struct UdpSocket {
    pub id: u32,
    pub local_port: u16,
    pub remote_port: u16,
    pub local_ip: [u8; 4],
    pub remote_ip: [u8; 4],
    pub bound: bool,
    pub connected: bool,
    pub rx_queue_head: AtomicU32,
    pub rx_queue_tail: AtomicU32,
}

impl Clone for UdpSocket {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            local_port: self.local_port.clone(),
            remote_port: self.remote_port.clone(),
            local_ip: self.local_ip.clone(),
            remote_ip: self.remote_ip.clone(),
            bound: self.bound.clone(),
            connected: self.connected.clone(),
            rx_queue_head: AtomicU32::new(self.rx_queue_head.load(core::sync::atomic::Ordering::Relaxed)),
            rx_queue_tail: AtomicU32::new(self.rx_queue_tail.load(core::sync::atomic::Ordering::Relaxed)),
        }
    }
}

impl UdpSocket {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            local_port: 0,
            remote_port: 0,
            local_ip: [0; 4],
            remote_ip: [0; 4],
            bound: false,
            connected: false,
            rx_queue_head: AtomicU32::new(0),
            rx_queue_tail: AtomicU32::new(0),
        }
    }

    pub fn bind(&mut self, port: u16, ip: [u8; 4]) {
        self.local_port = port;
        self.local_ip = ip;
        self.bound = true;
    }

    pub fn connect(&mut self, port: u16, ip: [u8; 4]) {
        self.remote_port = port;
        self.remote_ip = ip;
        self.connected = true;
    }
}

/// UDP ReceiveBuffer
pub struct UdpRxBuffer {
    pub data: [u8; 2048],
    pub len: usize,
    pub src_port: u16,
    pub src_ip: [u8; 4],
}

impl UdpRxBuffer {
    pub const fn new() -> Self {
        Self {
            data: [0; 2048],
            len: 0,
            src_port: 0,
            src_ip: [0; 4],
        }
    }
}

/// UDP Manager
pub struct UdpManager {
    sockets: [Option<UdpSocket>; 256],
    num_sockets: AtomicU32,
    next_socket_id: AtomicU32,
    next_port: AtomicU16,
}

impl UdpManager {
    pub const fn new() -> Self {
        Self {
            sockets: [const { None }; 256],
            num_sockets: AtomicU32::new(0),
            next_socket_id: AtomicU32::new(1),
            next_port: AtomicU16::new(49152),
        }
    }

    pub fn init(&mut self) {
        crate::log_info!("UDP manager initialized");
    }

    pub fn create_socket(&mut self) -> Option<u32> {
        let id = self.next_socket_id.fetch_add(1, Ordering::Relaxed);
        let idx = self.num_sockets.load(Ordering::Relaxed) as usize;
        
        if idx < 256 {
            self.sockets[idx] = Some(UdpSocket::new(id));
            self.num_sockets.fetch_add(1, Ordering::Relaxed);
            return Some(id);
        }
        None
    }

    pub fn bind_socket(&mut self, id: u32, port: u16, ip: [u8; 4]) -> bool {
        for i in 0..self.num_sockets.load(Ordering::Relaxed) as usize {
            if let Some(ref mut sock) = self.sockets[i] {
                if sock.id == id {
                    sock.bind(port, ip);
                    return true;
                }
            }
        }
        false
    }

    pub fn connect_socket(&mut self, id: u32, port: u16, ip: [u8; 4]) -> bool {
        for i in 0..self.num_sockets.load(Ordering::Relaxed) as usize {
            if let Some(ref mut sock) = self.sockets[i] {
                if sock.id == id {
                    sock.connect(port, ip);
                    return true;
                }
            }
        }
        false
    }

    pub fn allocate_port(&self) -> u16 {
        let port = self.next_port.fetch_add(1, Ordering::Relaxed);
        if port >= 65535 {
            self.next_port.store(49152, Ordering::Relaxed);
        }
        port
    }

    pub fn process_datagram(&mut self, header: &UdpHeader, payload: &[u8], src_ip: [u8; 4]) {
        // FindMatch suiteacceptWord
        for i in 0..self.num_sockets.load(Ordering::Relaxed) as usize {
            if let Some(ref sock) = self.sockets[i] {
                if sock.local_port == header.dst_port {
                    // willDatareleaseenterReceiveQueue
                    let _ = (header, payload, src_ip);
                    return;
                }
            }
        }
    }
}