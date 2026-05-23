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


use core::sync::atomic::{AtomicU16, AtomicU32, Ordering};
use crate::{pr_debug, pr_info};

/// TCP State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    /// Close
    Closed = 0,
    /// listen
    Listen = 1,
    /// SynchronousSend
    SynSent = 2,
    /// SynchronousReceive
    SynReceived = 3,
    /// alreadybuildcube
    Established = 4,
    /// WaitClose
    FinWait1 = 5,
    /// WaitClose 2
    FinWait2 = 6,
    /// CloseWait
    CloseWait = 7,
    /// Closeinfix
    Closing = 8,
    /// mostthenconfirm
    LastAck = 9,
    /// TimeWait
    TimeWait = 10,
}

/// TCP Flag
pub mod tcp_flags {
    pub const FIN: u8 = 0x01;
    pub const SYN: u8 = 0x02;
    pub const RST: u8 = 0x04;
    pub const PSH: u8 = 0x08;
    pub const ACK: u8 = 0x10;
    pub const URG: u8 = 0x20;
}

/// TCP Headpart
#[repr(C, packed)]
pub struct TcpHeader {
    /// sourcePort
    pub src_port: u16,
    /// targetPort
    pub dst_port: u16,
    /// Sequence number
    pub seq_num: u32,
    /// Acknowledgment number
    pub ack_num: u32,
    /// DataOffsetandFlag
    pub data_offset: u8,
    pub flags: u8,
    /// WindowSize
    pub window: u16,
    /// checksum
    pub checksum: u16,
    /// urgenturgentpointer
    pub urgent_ptr: u16,
}

/// TCP Join
pub struct TcpConnection {
    /// LocalPort
    pub local_port: AtomicU16,
    /// farprocessPort
    pub remote_port: AtomicU16,
    /// farprocess IP
    pub remote_ip: AtomicU32,
    /// CurrentState
    pub state: AtomicU32,
    /// Sequence number
    pub seq_num: AtomicU32,
    /// Acknowledgment number
    pub ack_num: AtomicU32,
    /// WindowSize
    pub window: AtomicU16,
}

impl Clone for TcpConnection {
    fn clone(&self) -> Self {
        Self {
            local_port: AtomicU16::new(self.local_port.load(core::sync::atomic::Ordering::Relaxed)),
            remote_port: AtomicU16::new(self.remote_port.load(core::sync::atomic::Ordering::Relaxed)),
            remote_ip: AtomicU32::new(self.remote_ip.load(core::sync::atomic::Ordering::Relaxed)),
            state: AtomicU32::new(self.state.load(core::sync::atomic::Ordering::Relaxed)),
            seq_num: AtomicU32::new(self.seq_num.load(core::sync::atomic::Ordering::Relaxed)),
            ack_num: AtomicU32::new(self.ack_num.load(core::sync::atomic::Ordering::Relaxed)),
            window: AtomicU16::new(self.window.load(core::sync::atomic::Ordering::Relaxed)),
        }
    }
}

impl TcpConnection {
    pub const fn new() -> Self {
        TcpConnection {
            local_port: AtomicU16::new(0),
            remote_port: AtomicU16::new(0),
            remote_ip: AtomicU32::new(0),
            state: AtomicU32::new(TcpState::Closed as u32),
            seq_num: AtomicU32::new(0),
            ack_num: AtomicU32::new(0),
            window: AtomicU16::new(65535),
        }
    }
    
    /// GetState
    pub fn get_state(&self) -> TcpState {
        match self.state.load(Ordering::Acquire) {
            0 => TcpState::Closed,
            1 => TcpState::Listen,
            2 => TcpState::SynSent,
            3 => TcpState::SynReceived,
            4 => TcpState::Established,
            5 => TcpState::FinWait1,
            6 => TcpState::FinWait2,
            7 => TcpState::CloseWait,
            8 => TcpState::Closing,
            9 => TcpState::LastAck,
            10 => TcpState::TimeWait,
            _ => TcpState::Closed,
        }
    }
    
    /// SetState
    pub fn set_state(&self, state: TcpState) {
        self.state.store(state as u32, Ordering::Release);
    }
}

/// TCP Service
pub struct TcpService {
    /// JoinArray
    connections: [Option<TcpConnection>; 64],
    /// Joincount
    num_connections: u32,
}

impl TcpService {
    pub const fn new() -> Self {
        TcpService {
            connections: [const { None }; 64],
            num_connections: 0,
        }
    }
    
    /// Initialize
    pub fn init(&mut self) -> i32 {
        log_info!("TCP service initialized");
        0
    }
    
    /// CreateJoin
    pub fn create_connection(&mut self) -> Option<u32> {
        for (i, slot) in self.connections.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(TcpConnection::new());
                self.num_connections += 1;
                return Some(i as u32);
            }
        }
        None
    }
    
    /// Jointofarprocess
    pub fn connect(&mut self, conn_id: u32, remote_ip: u32, remote_port: u16) -> i32 {
        let conn = match self.connections.get(conn_id as usize) {
            Some(c) => c,
            None => return -1, // EINVAL
        };
        if let Some(ref conn) = conn {
            conn.remote_ip.store(remote_ip, Ordering::Release);
            conn.remote_port.store(remote_port, Ordering::Release);
            conn.set_state(TcpState::SynSent);
            
            // TODO: Send SYN Package
            
            log_debug!("TCP connect to {:x}:{}", remote_ip, remote_port);
            return 0;
        }
        -1
    }
    
    /// SendData
    pub fn send(&self, conn_id: u32, data: &[u8]) -> i32 {
        let conn = match self.connections.get(conn_id as usize) {
            Some(c) => c,
            None => return -1, // EINVAL
        };
        if let Some(ref conn) = conn {
            if conn.get_state() != TcpState::Established {
                return -1;
            }
            
            // TODO: Send TCP DataPackage
            
            log_debug!("TCP send {} bytes on connection {}", data.len(), conn_id);
            return data.len() as i32;
        }
        -1
    }
    
    /// ReceiveData
    pub fn recv(&self, conn_id: u32, buf: &mut [u8]) -> i32 {
        let _conn = match self.connections.get(conn_id as usize) {
            Some(c) => c,
            None => return -1, // EINVAL
        };
        if let Some(ref _conn) = _conn {
            // TODO: Read from receive bufferData
            
            return 0;
        }
        -1
    }
    
    /// CloseJoin
    pub fn close(&mut self, conn_id: u32) -> i32 {
        let conn = match self.connections.get(conn_id as usize) {
            Some(c) => c,
            None => return -1, // EINVAL
        };
        if let Some(ref conn) = conn {
            conn.set_state(TcpState::FinWait1);
            
            // TODO: Send FIN Package
            
            return 0;
        }
        -1
    }
}

/// Global TCP Service
static mut TCP_SERVICE: TcpService = TcpService::new();

pub fn get_tcp_service() -> &'static mut TcpService {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &mut TCP_SERVICE }
}

pub fn init_tcp() {
    let service = get_tcp_service();
    service.init();
}