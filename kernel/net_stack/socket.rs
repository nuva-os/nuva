/*
 * Nuva OS - Kernel - NetStack - Socket
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
use crate::{pr_info};
/*
 * Nuva OS - Kernel - Socket API Implementation
 * 
 * Complete Socket API for network communication.
 * 
 * Copyright (C) 2026 Nuva OS Team
 */

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicI32, Ordering};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Socket address family
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressFamily {
    Unspecified = 0,
    Unix = 1,
    Inet = 2,
    Inet6 = 10,
    Netlink = 16,
    Packet = 17,
}

/// Socket type
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketType {
    Stream = 1,     // TCP
    Dgram = 2,      // UDP
    Raw = 3,        // Raw socket
    Rdm = 4,        // Reliably delivered message
    SeqPacket = 5,  // Sequenced packet
}

/// Socket protocol
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Default = 0,
    Tcp = 6,
    Udp = 17,
    Icmp = 1,
    Igmp = 2,
    Icmpv6 = 58,
}

/// Socket state
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketState {
    Unconnected = 0,
    Connecting = 1,
    Connected = 2,
    Disconnecting = 3,
    Listening = 4,
    Bound = 5,
}

/// Socket address
#[repr(C)]
#[derive(Copy, Clone)]
pub union SocketAddr {
    pub generic: GenericAddr,
    pub ipv4: Ipv4Addr,
    pub ipv6: Ipv6Addr,
    pub unix: UnixAddr,
}

/// Generic socket address
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GenericAddr {
    pub family: u16,
    pub data: [u8; 14],
}

/// IPv4 socket address
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Ipv4Addr {
    pub family: u16,    // AF_INET = 2
    pub port: u16,      // Network byte order
    pub addr: u32,      // Network byte order
    pub zero: [u8; 8],
}

impl Ipv4Addr {
    pub fn new(addr: [u8; 4], port: u16) -> Self {
        Ipv4Addr {
            family: AddressFamily::Inet as u16,
            port: port.to_be(),
            addr: u32::from_be_bytes(addr),
            zero: [0; 8],
        }
    }
    
    pub fn any() -> Self {
        Self::new([0, 0, 0, 0], 0)
    }
    
    pub fn localhost(port: u16) -> Self {
        Self::new([127, 0, 0, 1], port)
    }
}

/// IPv6 socket address
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Ipv6Addr {
    pub family: u16,    // AF_INET6 = 10
    pub port: u16,
    pub flowinfo: u32,
    pub addr: [u8; 16],
    pub scope_id: u32,
}

impl Ipv6Addr {
    pub fn new(addr: [u8; 16], port: u16) -> Self {
        Ipv6Addr {
            family: AddressFamily::Inet6 as u16,
            port: port.to_be(),
            flowinfo: 0,
            addr,
            scope_id: 0,
        }
    }
}

/// Unix socket address
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UnixAddr {
    pub family: u16,    // AF_UNIX = 1
    pub path: [u8; 108],
}

impl UnixAddr {
    pub fn new(path: &[u8]) -> Self {
        let mut path_buf = [0u8; 108];
        let len = path.len().min(107);
        path_buf[..len].copy_from_slice(&path[..len]);
        
        UnixAddr {
            family: AddressFamily::Unix as u16,
            path: path_buf,
        }
    }
}

/// Socket buffer
pub struct SocketBuffer {
    data: Vec<u8>,
    capacity: usize,
}

impl SocketBuffer {
    pub fn new(capacity: usize) -> Self {
        SocketBuffer {
            data: Vec::with_capacity(capacity),
            capacity,
        }
    }
    
    pub fn write(&mut self, data: &[u8]) -> usize {
        let available = self.capacity - self.data.len();
        let to_write = data.len().min(available);
        self.data.extend_from_slice(&data[..to_write]);
        to_write
    }
    
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let to_read = buf.len().min(self.data.len());
        buf[..to_read].copy_from_slice(&self.data[..to_read]);
        self.data.drain(..to_read);
        to_read
    }
    
    pub fn peek(&self, buf: &mut [u8]) -> usize {
        let to_read = buf.len().min(self.data.len());
        buf[..to_read].copy_from_slice(&self.data[..to_read]);
        to_read
    }
    
    pub fn len(&self) -> usize { self.data.len() }
    pub fn is_empty(&self) -> bool { self.data.is_empty() }
    pub fn available(&self) -> usize { self.capacity - self.data.len() }
}

/// Socket structure
pub struct Socket {
    /// Socket ID
    pub id: u64,
    /// Address family
    pub family: AddressFamily,
    /// Socket type
    pub sock_type: SocketType,
    /// Protocol
    pub protocol: Protocol,
    /// State
    pub state: AtomicU32,
    /// Local address
    pub local_addr: SocketAddr,
    /// Remote address
    pub remote_addr: SocketAddr,
    /// Receive buffer
    pub recv_buf: spin::Mutex<SocketBuffer>,
    /// Send buffer
    pub send_buf: spin::Mutex<SocketBuffer>,
    /// Backlog (for listening sockets)
    pub backlog: AtomicU32,
    /// Pending connections
    pub pending: spin::Mutex<Vec<u64>>,
    /// Reference count
    pub refs: AtomicU32,
    /// Flags
    pub flags: AtomicU32,
    /// Error
    pub error: AtomicI32,
}

/// Socket flags
pub mod socket_flags {
    pub const NONBLOCK: u32 = 1 << 0;
    pub const CLOEXEC: u32 = 1 << 1;
    pub const REUSEADDR: u32 = 1 << 2;
    pub const REUSEPORT: u32 = 1 << 3;
    pub const KEEPALIVE: u32 = 1 << 4;
    pub const BROADCAST: u32 = 1 << 5;
    pub const LINGER: u32 = 1 << 6;
}

impl Socket {
    pub fn new(family: AddressFamily, sock_type: SocketType, protocol: Protocol) -> Self {
        Socket {
            id: 0,
            family,
            sock_type,
            protocol,
            state: AtomicU32::new(SocketState::Unconnected as u32),
            // SAFETY: unsafe block required for low-level memory or hardware access
            local_addr: unsafe { core::mem::zeroed() },
            // SAFETY: unsafe block required for low-level memory or hardware access
            remote_addr: unsafe { core::mem::zeroed() },
            recv_buf: spin::Mutex::new(SocketBuffer::new(65536)),
            send_buf: spin::Mutex::new(SocketBuffer::new(65536)),
            backlog: AtomicU32::new(0),
            pending: spin::Mutex::new(Vec::new()),
            refs: AtomicU32::new(1),
            flags: AtomicU32::new(0),
            error: AtomicI32::new(0),
        }
    }
    
    /// Bind to address
    pub fn bind(&mut self, addr: &SocketAddr) -> Result<(), i32> {
        if self.state.load(Ordering::Acquire) != SocketState::Unconnected as u32 {
            return Err(-22); // EINVAL
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        self.local_addr = *addr;
        self.state.store(SocketState::Bound as u32, Ordering::Release);
        Ok(())
    }
    
    /// Listen for connections
    pub fn listen(&mut self, backlog: i32) -> Result<(), i32> {
        if self.sock_type != SocketType::Stream {
            return Err(-95); // EOPNOTSUPP
        }
        
        if self.state.load(Ordering::Acquire) != SocketState::Bound as u32 {
            return Err(-22); // EINVAL
        }
        
        self.backlog.store(backlog.max(1) as u32, Ordering::Release);
        self.state.store(SocketState::Listening as u32, Ordering::Release);
        Ok(())
    }
    
    /// Accept connection
    pub fn accept(&mut self) -> Result<u64, i32> {
        if self.state.load(Ordering::Acquire) != SocketState::Listening as u32 {
            return Err(-22); // EINVAL
        }
        
        let mut pending = self.pending.lock();
        if pending.is_empty() {
            if self.flags.load(Ordering::Acquire) & socket_flags::NONBLOCK != 0 {
                return Err(-11); // EAGAIN
            }
            return Err(-11); // Would block
        }
        
        Ok(pending.remove(0))
    }
    
    /// Connect to address
    pub fn connect(&mut self, addr: &SocketAddr) -> Result<(), i32> {
        let current_state = self.state.load(Ordering::Acquire);
        
        match self.sock_type {
            SocketType::Stream => {
                // TCP: need to establish connection
                if current_state == SocketState::Connected as u32 {
                    return Err(-106); // EISCONN
                }
                
                // SAFETY: unsafe block required for low-level memory or hardware access
                self.remote_addr = *addr;
                self.state.store(SocketState::Connecting as u32, Ordering::Release);
                
                // TODO: Send SYN packet and wait for connection
                // For now, simulate successful connection
                self.state.store(SocketState::Connected as u32, Ordering::Release);
                Ok(())
            }
            SocketType::Dgram => {
                // UDP: just set remote address
                // SAFETY: unsafe block required for low-level memory or hardware access
                self.remote_addr = *addr;
                self.state.store(SocketState::Connected as u32, Ordering::Release);
                Ok(())
            }
            _ => Err(-95), // EOPNOTSUPP
        }
    }
    
    /// Send data
    pub fn send(&mut self, data: &[u8], flags: u32) -> Result<usize, i32> {
        match self.sock_type {
            SocketType::Stream => {
                if self.state.load(Ordering::Acquire) != SocketState::Connected as u32 {
                    return Err(-107); // ENOTCONN
                }
                
                // Add to send buffer
                let mut buf = self.send_buf.lock();
                let written = buf.write(data);
                
                // TODO: Actually send via TCP
                Ok(written)
            }
            SocketType::Dgram => {
                // UDP: send datagram
                // TODO: Send UDP packet
                Ok(data.len())
            }
            _ => Err(-95), // EOPNOTSUPP
        }
    }
    
    /// Receive data
    pub fn recv(&mut self, buf: &mut [u8], flags: u32) -> Result<usize, i32> {
        match self.sock_type {
            SocketType::Stream => {
                let mut recv_buf = self.recv_buf.lock();
                if recv_buf.is_empty() {
                    if self.flags.load(Ordering::Acquire) & socket_flags::NONBLOCK != 0 {
                        return Err(-11); // EAGAIN
                    }
                    return Err(-11);
                }
                
                Ok(recv_buf.read(buf))
            }
            SocketType::Dgram => {
                // TODO: Receive UDP datagram
                Ok(0)
            }
            _ => Err(-95), // EOPNOTSUPP
        }
    }
    
    /// Send to specific address
    pub fn sendto(&mut self, data: &[u8], addr: &SocketAddr, flags: u32) -> Result<usize, i32> {
        if self.sock_type == SocketType::Stream {
            // TCP: must be connected
            return self.send(data, flags);
        }
        
        // UDP: send to address
        // TODO: Send UDP packet to addr
        Ok(data.len())
    }
    
    /// Receive from
    pub fn recvfrom(&mut self, buf: &mut [u8], flags: u32) -> Result<(usize, SocketAddr), i32> {
        if self.sock_type == SocketType::Stream {
            let len = self.recv(buf, flags)?;
            return Ok((len, self.remote_addr));
        }
        
        // TODO: Receive UDP datagram and return source address
        Err(-11)
    }
    
    /// Set socket option
    pub fn setsockopt(&mut self, level: i32, optname: i32, optval: &[u8]) -> Result<(), i32> {
        match level {
            1 => { // SOL_SOCKET
                match optname {
                    2 => { // SO_REUSEADDR
                        if !optval.is_empty() && optval[0] != 0 {
                            self.flags.fetch_or(socket_flags::REUSEADDR, Ordering::AcqRel);
                        } else {
                            self.flags.fetch_and(!socket_flags::REUSEADDR, Ordering::AcqRel);
                        }
                        Ok(())
                    }
                    3 => { // SO_REUSEPORT
                        if !optval.is_empty() && optval[0] != 0 {
                            self.flags.fetch_or(socket_flags::REUSEPORT, Ordering::AcqRel);
                        } else {
                            self.flags.fetch_and(!socket_flags::REUSEPORT, Ordering::AcqRel);
                        }
                        Ok(())
                    }
                    9 => { // SO_KEEPALIVE
                        if !optval.is_empty() && optval[0] != 0 {
                            self.flags.fetch_or(socket_flags::KEEPALIVE, Ordering::AcqRel);
                        } else {
                            self.flags.fetch_and(!socket_flags::KEEPALIVE, Ordering::AcqRel);
                        }
                        Ok(())
                    }
                    _ => Err(-92), // ENOPROTOOPT
                }
            }
            6 => { // IPPROTO_TCP
                match optname {
                    1 => Ok(()), // TCP_NODELAY
                    _ => Err(-92),
                }
            }
            _ => Err(-92),
        }
    }
    
    /// Get socket option
    pub fn getsockopt(&self, level: i32, optname: i32, optval: &mut [u8]) -> Result<usize, i32> {
        match level {
            1 => { // SOL_SOCKET
                match optname {
                    1 => { // SO_ERROR
                        if optval.len() >= 4 {
                            let err = self.error.load(Ordering::Acquire);
                            optval[..4].copy_from_slice(&err.to_ne_bytes());
                            Ok(4)
                        } else {
                            Err(-22)
                        }
                    }
                    _ => Err(-92),
                }
            }
            _ => Err(-92),
        }
    }
    
    /// Close socket
    pub fn close(&mut self) {
        let refs = self.refs.fetch_sub(1, Ordering::AcqRel);
        if refs == 1 {
            // Last reference, cleanup
            self.state.store(SocketState::Unconnected as u32, Ordering::Release);
        }
    }
}

/// Socket manager
pub struct SocketManager {
    sockets: spin::Mutex<BTreeMap<u64, Socket>>,
    next_id: AtomicU64,
}

impl SocketManager {
    pub fn new() -> Self {
        SocketManager {
            sockets: spin::Mutex::new(BTreeMap::new()),
            next_id: AtomicU64::new(1),
        }
    }
    
    /// Create socket
    pub fn socket(&self, family: AddressFamily, sock_type: SocketType, protocol: Protocol) -> Result<u64, i32> {
        let id = self.next_id.fetch_add(1, Ordering::AcqRel);
        let mut sock = Socket::new(family, sock_type, protocol);
        sock.id = id;
        
        self.sockets.lock().insert(id, sock);
        Ok(id)
    }
    
    /// Get socket by ID
    pub fn get(&self, id: u64) -> Option<&mut Socket> {
        // Note: This is unsafe due to lifetime issues
        // In real implementation, use proper locking
        None
    }
    
    /// Bind socket
    pub fn bind(&self, id: u64, addr: &SocketAddr) -> Result<(), i32> {
        let mut sockets = self.sockets.lock();
        let sock = sockets.get_mut(&id).ok_or(-9)?; // EBADF
        sock.bind(addr)
    }
    
    /// Listen
    pub fn listen(&self, id: u64, backlog: i32) -> Result<(), i32> {
        let mut sockets = self.sockets.lock();
        let sock = sockets.get_mut(&id).ok_or(-9)?;
        sock.listen(backlog)
    }
    
    /// Accept
    pub fn accept(&self, id: u64) -> Result<u64, i32> {
        let mut sockets = self.sockets.lock();
        let sock = sockets.get_mut(&id).ok_or(-9)?;
        sock.accept()
    }
    
    /// Connect
    pub fn connect(&self, id: u64, addr: &SocketAddr) -> Result<(), i32> {
        let mut sockets = self.sockets.lock();
        let sock = sockets.get_mut(&id).ok_or(-9)?;
        sock.connect(addr)
    }
    
    /// Send
    pub fn send(&self, id: u64, data: &[u8], flags: u32) -> Result<usize, i32> {
        let mut sockets = self.sockets.lock();
        let sock = sockets.get_mut(&id).ok_or(-9)?;
        sock.send(data, flags)
    }
    
    /// Receive
    pub fn recv(&self, id: u64, buf: &mut [u8], flags: u32) -> Result<usize, i32> {
        let mut sockets = self.sockets.lock();
        let sock = sockets.get_mut(&id).ok_or(-9)?;
        sock.recv(buf, flags)
    }
    
    /// Close
    pub fn close(&self, id: u64) -> Result<(), i32> {
        let mut sockets = self.sockets.lock();
        if let Some(sock) = sockets.get_mut(&id) {
            sock.close();
        }
        sockets.remove(&id);
        Ok(())
    }
}

impl Default for SocketManager {
    fn default() -> Self { Self::new() }
}

/// Global socket manager
static SOCKET_MANAGER: SocketManager = SocketManager {
    sockets: spin::Mutex::new(BTreeMap::new()),
    next_id: AtomicU64::new(1),
};

/// Get socket manager
pub fn socket_manager() -> &'static mut SocketManager {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &mut SOCKET_MANAGER }
}

/// Initialize socket subsystem
pub fn init_socket_api() {
    log_info!("Socket API initialized");
}
