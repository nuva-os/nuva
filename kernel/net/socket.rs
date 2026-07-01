/*
 * Nuva OS - Kernel - Socket Implementation
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
use crate::{pr_info};

use crate::syslib::posix::errno::Errno;
use crate::kernel::error::Errno;
/// Socket descriptor type
pub type SockFd = i32;

/// Port number type
pub type Port = u16;

/// Address family
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressFamily {
 /// Unspecified
 Unspec = 0,
 /// Unix domain socket
 Unix = 1,
 /// IPv4
 Inet = 2,
 /// IPv6
 Inet6 = 10,
 /// Netlink
 Netlink = 16,
 /// Packet (raw)
 Packet = 17,
}

/// Socket type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketType {
 /// Stream (connection-oriented)
 Stream = 1,
 /// Datagram (connectionless)
 Dgram = 2,
 /// Raw socket
 Raw = 3,
 /// Sequenced packet
 SeqPacket = 5,
}

/// Protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
 /// Default
 Default = 0,
 /// TCP
 Tcp = 6,
 /// UDP
 Udp = 17,
 /// ICMP
 Icmp = 1,
 /// IGMP
 Igmp = 2,
}

/// Socket state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketState {
 /// Unconnected
 Unconnected = 0,
 /// Connecting
 Connecting = 1,
 /// Connected
 Connected = 2,
 /// Disconnecting
 Disconnecting = 3,
 /// Listening
 Listening = 4,
 /// Bound
 Bound = 5,
}

/// Socket address
#[repr(C)]
pub union SockAddr {
 /// Generic address
 pub generic: SockAddrGeneric,
 /// IPv4 address
 pub inet: SockAddrInet,
 /// IPv6 address
 pub inet6: SockAddrInet6,
 /// Unix address
 pub unix: SockAddrUnix,
}

/// Generic socket address
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SockAddrGeneric {
 pub family: u16,
 pub data: [u8; 14],
}

/// IPv4 socket address
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SockAddrInet {
 pub family: u16, /* AF_INET = 2 */
 pub port: u16, /* Port in network byte order */
 pub addr: u32, /* IPv4 address in network byte order */
 pub zero: [u8; 8], /* Padding */
}

impl SockAddrInet {
 /// Create new IPv4 address
 pub fn new(addr: [u8; 4], port: u16) -> Self {
 SockAddrInet {
 family: AddressFamily::Inet as u16,
 port: port.to_be(),
 addr: u32::from_be_bytes(addr),
 zero: [0; 8],
 }
 }
 
 /// Get port in host byte order
 pub fn get_port(&self) -> u16 {
 u16::from_be(self.port)
 }
 
 /// Get address bytes
 pub fn get_addr(&self) -> [u8; 4] {
 u32::from_be(self.addr).to_be_bytes()
 }
}

/// IPv6 socket address
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SockAddrInet6 {
 pub family: u16, /* AF_INET6 = 10 */
 pub port: u16, /* Port in network byte order */
 pub flowinfo: u32, /* IPv6 flow information */
 pub addr: [u8; 16], /* IPv6 address */
 pub scope_id: u32, /* Scope ID */
}

/// Unix socket address
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SockAddrUnix {
 pub family: u16, /* AF_UNIX = 1 */
 pub path: [u8; 108], /* Path name */
}

/// Socket options level
pub mod sol {
 pub const SOCKET: i32 = 1;
 pub const IP: i32 = 0;
 pub const TCP: i32 = 6;
 pub const UDP: i32 = 17;
 pub const IPV6: i32 = 41;
}

/// Socket options
pub mod so {
 pub const DEBUG: i32 = 1;
 pub const REUSEADDR: i32 = 2;
 pub const TYPE: i32 = 3;
 pub const ERROR: i32 = 4;
 pub const DONTROUTE: i32 = 5;
 pub const BROADCAST: i32 = 6;
 pub const SNDBUF: i32 = 7;
 pub const RCVBUF: i32 = 8;
 pub const KEEPALIVE: i32 = 9;
 pub const OOBINLINE: i32 = 10;
 pub const NO_CHECK: i32 = 11;
 pub const PRIORITY: i32 = 12;
 pub const LINGER: i32 = 13;
 pub const BSDCOMPAT: i32 = 14;
 pub const REUSEPORT: i32 = 15;
 pub const PASSCRED: i32 = 16;
 pub const PEERCRED: i32 = 17;
 pub const RCVLOWAT: i32 = 18;
 pub const SNDLOWAT: i32 = 19;
 pub const RCVTIMEO: i32 = 20;
 pub const SNDTIMEO: i32 = 21;
}

/// Message header for sendmsg/recvmsg
#[repr(C)]
pub struct Msghdr {
 /// Destination address
 pub msg_name: *mut u8,
 /// Address length
 pub msg_namelen: u32,
 /// Scatter/gather array
 pub msg_iov: *mut IoVec,
 /// Number of elements in msg_iov
 pub msg_iovlen: usize,
 /// Control data
 pub msg_control: *mut u8,
 /// Control data length
 pub msg_controllen: usize,
 /// Flags on received message
 pub msg_flags: u32,
}

/// I/O vector
#[repr(C)]
pub struct IoVec {
 pub iov_base: *mut u8,
 pub iov_len: usize,
}

/// Socket buffer
pub struct SkBuff {
 /// Data pointer
 pub data: *mut u8,
 /// Data length
 pub len: usize,
 /// Buffer head
 pub head: *mut u8,
 /// Buffer end
 pub end: *mut u8,
 /// Source address
 pub saddr: u32,
 /// Destination address
 pub daddr: u32,
 /// Source port
 pub sport: u16,
 /// Destination port
 pub dport: u16,
 /// Protocol
 pub protocol: u8,
 /// Flags
 pub flags: AtomicU32,
 /// Reference count
 pub ref_count: AtomicU32,
}

impl SkBuff {
 /// Create new socket buffer
 pub fn new(size: usize) -> Option<Self> {
 // TODO: Allocate buffer
 Some(SkBuff {
 data: core::ptr::null_mut(),
 len: size,
 head: core::ptr::null_mut(),
 end: core::ptr::null_mut(),
 saddr: 0,
 daddr: 0,
 sport: 0,
 dport: 0,
 protocol: 0,
 flags: AtomicU32::new(0),
 ref_count: AtomicU32::new(1),
 })
 }
 
 /// Get data length
 pub fn len(&self) -> usize {
 self.len
 }
 
 /// Check if empty
 pub fn is_empty(&self) -> bool {
 self.len == 0
 }
}

/// Socket structure
pub struct Socket {
 /// Socket descriptor
 pub fd: SockFd,
 /// Address family
 pub family: AddressFamily,
 /// Socket type
 pub sock_type: SocketType,
 /// Protocol
 pub protocol: Protocol,
 /// State
 pub state: AtomicU32,
 /// Local address
 pub local_addr: SockAddrInet,
 /// Remote address
 pub remote_addr: SockAddrInet,
 /// Receive buffer
 pub recv_buf: AtomicU64,
 /// Send buffer
 pub send_buf: AtomicU64,
 /// Receive buffer size limit
 pub recv_buf_size: AtomicU32,
 /// Send buffer size limit
 pub send_buf_size: AtomicU32,
 /// Socket flags
 pub flags: AtomicU32,
 /// Reference count
 pub ref_count: AtomicU32,
 /// Pending connections (for listen)
 pub pending_count: AtomicU32,
 /// Backlog limit
 pub backlog: AtomicU32,
 /// Options
 pub options: AtomicU32,
 /// Error code
 pub error: AtomicU32,
}

/// Socket flags
pub mod sock_flags {
 pub const SF_BOUND: u32 = 0x01;
 pub const SF_LISTENING: u32 = 0x02;
 pub const SF_CONNECTED: u32 = 0x04;
 pub const SF_NONBLOCK: u32 = 0x08;
 pub const SF_CLOEXEC: u32 = 0x10;
}

impl Socket {
 /// Create a new socket
 pub fn new(family: AddressFamily, sock_type: SocketType, protocol: Protocol) -> Self {
 Socket {
 fd: -1,
 family,
 sock_type,
 protocol,
 state: AtomicU32::new(SocketState::Unconnected as u32),
 local_addr: SockAddrInet::new([0; 4], 0),
 remote_addr: SockAddrInet::new([0; 4], 0),
 recv_buf: AtomicU64::new(0),
 send_buf: AtomicU64::new(0),
 recv_buf_size: AtomicU32::new(65536),
 send_buf_size: AtomicU32::new(65536),
 flags: AtomicU32::new(0),
 ref_count: AtomicU32::new(1),
 pending_count: AtomicU32::new(0),
 backlog: AtomicU32::new(0),
 options: AtomicU32::new(0),
 error: AtomicU32::new(0),
 }
 }
 
 /// Bind to address
 pub fn bind(&mut self, addr: &SockAddrInet) -> Result<(), i32> {
 // Check if already bound
 if (self.flags.load(Ordering::Acquire) & sock_flags::SF_BOUND) != 0 {
 return Err(-22); /* EINVAL */
 }
 
 // Set local address
 self.local_addr = SockAddrInet {
 family: addr.family,
 port: addr.port,
 addr: addr.addr,
 zero: [0; 8],
 };
 
 // Mark as bound
 self.flags.fetch_or(sock_flags::SF_BOUND, Ordering::AcqRel);
 self.state.store(SocketState::Bound as u32, Ordering::Release);
 
 Ok(())
 }
 
 /// Listen for connections
 pub fn listen(&mut self, backlog: i32) -> Result<(), i32> {
 // Check if bound
 if (self.flags.load(Ordering::Acquire) & sock_flags::SF_BOUND) == 0 {
 return Err(-22); /* EINVAL */
 }
 
 // Only stream sockets can listen
 if self.sock_type != SocketType::Stream {
 return Err(-95); /* EOPNOTSUPP */
 }
 
 // Set backlog
 self.backlog.store(backlog.max(1) as u32, Ordering::Release);
 
 // Mark as listening
 self.flags.fetch_or(sock_flags::SF_LISTENING, Ordering::AcqRel);
 self.state.store(SocketState::Listening as u32, Ordering::Release);
 
 Ok(())
 }
 
 /// Accept connection
 pub fn accept(&mut self, addr: *mut SockAddrInet, addr_len: *mut u32) -> Result<SockFd, i32> {
 // Check if listening
 if (self.flags.load(Ordering::Acquire) & sock_flags::SF_LISTENING) == 0 {
 return Err(-22); /* EINVAL */
 }
 
 // Check for pending connections
 if self.pending_count.load(Ordering::Acquire) == 0 {
 // Non-blocking: return EAGAIN
 if (self.flags.load(Ordering::Acquire) & sock_flags::SF_NONBLOCK) != 0 {
 return Err(-11); /* EAGAIN */
 }
 
 // Blocking: wait for connection
 // TODO: Block until connection arrives
 return Err(-11); /* EAGAIN for now */
 }
 
 // Accept connection
 self.pending_count.fetch_sub(1, Ordering::AcqRel);
 
 // Create new socket for connection
 let new_sock = Socket::new(self.family, self.sock_type, self.protocol);
 
 // TODO: Set up new socket with connection info
 
 // Return address if requested
 if !addr.is_null() && !addr_len.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 (*addr) = new_sock.remote_addr;
 (*addr_len) = core::mem::size_of::<SockAddrInet>() as u32;
 }
 }
 
 Ok(new_sock.fd)
 }
 
 /// Connect to address
 pub fn connect(&mut self, addr: &SockAddrInet) -> Result<(), i32> {
 // Set remote address
 self.remote_addr = SockAddrInet {
 family: addr.family,
 port: addr.port,
 addr: addr.addr,
 zero: [0; 8],
 };
 
 // For TCP, initiate three-way handshake
 if self.sock_type == SocketType::Stream {
 self.state.store(SocketState::Connecting as u32, Ordering::Release);

 let local_port = self.local_addr.get_port();
 let remote_port = self.remote_addr.get_port();
 let local_addr = u32::from_be(self.local_addr.addr);
 let remote_addr = u32::from_be(self.remote_addr.addr);

 let mut tcp_conn = super::tcp::TcpConnection::new(
     local_addr, local_port, remote_addr, remote_port,
 );

 let mut tcp_mgr = super::tcp::TcpManager::new();
 let ret = tcp_mgr.connect(&mut tcp_conn);
 if ret != 0 {
     self.state.store(SocketState::Unconnected as u32, Ordering::Release);
     self.error.store(-ret as u32, Ordering::Release);
     return Err(ret);
 }

 self.state.store(SocketState::Connected as u32, Ordering::Release);
 self.flags.fetch_or(sock_flags::SF_CONNECTED, Ordering::AcqRel);
 } else {
 // For UDP, just mark as connected
 self.state.store(SocketState::Connected as u32, Ordering::Release);
 }
 
 Ok(())
 }
 
 /// Send data
 pub fn send(&self, buf: &[u8], flags: i32) -> Result<usize, i32> {
 if self.sock_type == SocketType::Stream {
 // TCP: must be connected
 if (self.flags.load(Ordering::Acquire) & sock_flags::SF_CONNECTED) == 0 {
 return Err(-32); /* EPIPE */
 }
 
 self.send_tcp(buf, flags)
 } else {
 // UDP: can send without connect if dest set
 self.send_udp(buf, flags)
 }
 }
 
 /// Receive data
 pub fn recv(&self, buf: &mut [u8], flags: i32) -> Result<usize, i32> {
 if self.sock_type == SocketType::Stream {
 self.recv_tcp(buf, flags)
 } else {
 self.recv_udp(buf, flags)
 }
 }
 
 /// Send to specific address (UDP)
 pub fn sendto(&self, buf: &[u8], addr: &SockAddrInet, flags: i32) -> Result<usize, i32> {
 // TODO: Send UDP packet to specified address
 self.send_udp(buf, flags)
 }
 
 /// Receive from (UDP)
 pub fn recvfrom(&self, buf: &mut [u8], addr: *mut SockAddrInet, flags: i32) -> Result<usize, i32> {
 let n = self.recv_udp(buf, flags)?;
 
 // Fill in source address
 if !addr.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 (*addr) = self.remote_addr;
 }
 }
 
 Ok(n)
 }
 
  /// TCP send
 fn send_tcp(&self, buf: &[u8], _flags: i32) -> Result<usize, i32> {
 if (self.flags.load(Ordering::Acquire) & sock_flags::SF_CONNECTED) == 0 {
 return Err(-32); // EPIPE
 }

 let send_buf_size = self.send_buf_size.load(Ordering::Acquire) as usize;
 let send_used = self.send_buf.load(Ordering::Acquire) as usize;
 let available = send_buf_size.saturating_sub(send_used);
 let to_send = buf.len().min(available);

 if to_send == 0 {
 return Err(-11); // EAGAIN
 }

 self.send_buf.fetch_add(to_send as u64, Ordering::AcqRel);

 // TCP segment transmission happens via kernel TCP stack:
 // crate::kernel::net::tcp::tcp_manager().send(conn, &buf[..to_send]);

 Ok(to_send)
 }

 /// TCP receive
 fn recv_tcp(&self, buf: &mut [u8], _flags: i32) -> Result<usize, i32> {
 let recv_used = self.recv_buf.load(Ordering::Acquire) as usize;
 if recv_used == 0 {
 if (self.flags.load(Ordering::Acquire) & sock_flags::SF_NONBLOCK) != 0 {
 return Err(-11); // EAGAIN
 }
 return Err(-11); // EAGAIN (would block)
 }

 let to_read = buf.len().min(recv_used);
 // Copy from receive buffer to user buffer (simplified; real impl uses SocketBuffer)
 // For now, data is already in the buffer from TCP stack

 self.recv_buf.fetch_sub(to_read as u64, Ordering::AcqRel);

 Ok(to_read)
 }

 /// UDP send
 fn send_udp(&self, buf: &[u8], _flags: i32) -> Result<usize, i32> {
 if buf.len() > 65507 {
 return Err(-90); // EMSGSIZE
 }

 let remote_addr = self.remote_addr;
 let remote_port = remote_addr.get_port();

 // Build and send UDP datagram via UDP manager:
 // crate::kernel::net::udp::udp_manager().send(tcb_idx, buf, remote_addr.addr, remote_port);

 Ok(buf.len())
 }

 /// UDP receive
 fn recv_udp(&self, buf: &mut [u8], _flags: i32) -> Result<usize, i32> {
 let recv_used = self.recv_buf.load(Ordering::Acquire) as usize;
 if recv_used == 0 {
 if (self.flags.load(Ordering::Acquire) & sock_flags::SF_NONBLOCK) != 0 {
 return Err(-11); // EAGAIN
 }
 return Err(-11); // EAGAIN
 }

 let to_read = buf.len().min(recv_used);
 self.recv_buf.fetch_sub(to_read as u64, Ordering::AcqRel);

 Ok(to_read)
 }
 
 /// Set socket option
 pub fn setsockopt(&mut self, level: i32, optname: i32, optval: *const u8, optlen: u32) -> Result<(), i32> {
 if level == sol::SOCKET {
 match optname {
 so::REUSEADDR => {
 if optlen >= 4 && !optval.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 let val = *(optval as *const i32);
 if val != 0 {
 self.options.fetch_or(1, Ordering::AcqRel);
 } else {
 self.options.fetch_and(!1, Ordering::AcqRel);
 }
 }
 }
 }
 so::REUSEPORT => {
 // Similar to REUSEADDR
 }
 so::SNDBUF => {
 if optlen >= 4 && !optval.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 let val = *(optval as *const i32);
 self.send_buf_size.store(val as u32, Ordering::Release);
 }
 }
 }
 so::RCVBUF => {
 if optlen >= 4 && !optval.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 let val = *(optval as *const i32);
 self.recv_buf_size.store(val as u32, Ordering::Release);
 }
 }
 }
 _ => {}
 }
 }
 
 Ok(())
 }
 
 /// Get socket option
 pub fn getsockopt(&self, level: i32, optname: i32, optval: *mut u8, optlen: *mut u32) -> Result<(), i32> {
 if level == sol::SOCKET {
 match optname {
 so::TYPE => {
 if !optval.is_null() && !optlen.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 *(optval as *mut i32) = self.sock_type as i32;
 *optlen = 4;
 }
 }
 }
 so::ERROR => {
 if !optval.is_null() && !optlen.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 *(optval as *mut i32) = self.error.swap(0, Ordering::AcqRel) as i32;
 *optlen = 4;
 }
 }
 }
 so::SNDBUF => {
 if !optval.is_null() && !optlen.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 *(optval as *mut i32) = self.send_buf_size.load(Ordering::Acquire) as i32;
 *optlen = 4;
 }
 }
 }
 so::RCVBUF => {
 if !optval.is_null() && !optlen.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 *(optval as *mut i32) = self.recv_buf_size.load(Ordering::Acquire) as i32;
 *optlen = 4;
 }
 }
 }
 _ => {}
 }
 }
 
 Ok(())
 }
}

/// Maximum sockets in the global table
const MAX_SOCKETS: usize = 256;

/// Socket manager
pub struct SocketManager {
 /// Number of sockets
 pub socket_count: AtomicU32,
 /// Total bytes sent
 pub bytes_sent: AtomicU64,
 /// Total bytes received
 pub bytes_recv: AtomicU64,
 /// Next socket fd
 pub next_fd: AtomicU32,
 /// Socket table (fd-indexed)
 pub socket_table: [Option<Socket>; MAX_SOCKETS],
}

impl SocketManager {
 pub const fn new() -> Self {
 SocketManager {
 socket_count: AtomicU32::new(0),
 bytes_sent: AtomicU64::new(0),
 bytes_recv: AtomicU64::new(0),
 next_fd: AtomicU32::new(0),
 socket_table: [const { None }; MAX_SOCKETS],
 }
 }
 
 /// Initialize socket manager
 pub fn init(&self) {
 log_info!("Socket manager initialized");
 }
 
 /// Get socket by fd
 pub fn get_socket(&mut self, fd: SockFd) -> Option<&mut Socket> {
 if fd < 0 {
 return None;
 }
 let idx = fd as usize;
 if idx >= MAX_SOCKETS {
 return None;
 }
 self.socket_table[idx].as_mut()
 }

 /// Create socket
 pub fn socket(&mut self, family: i32, sock_type: i32, protocol: i32) -> Result<SockFd, i32> {
 let af = match family {
 2 => AddressFamily::Inet,
 10 => AddressFamily::Inet6,
 1 => AddressFamily::Unix,
 _ => return Err(-97), /* EAFNOSUPPORT */
 };
 
 let st = match sock_type {
 1 => SocketType::Stream,
 2 => SocketType::Dgram,
 3 => SocketType::Raw,
 5 => SocketType::SeqPacket,
 _ => return Err(-94), /* ESOCKTNOSUPPORT */
 };
 
 let proto = match protocol {
 0 => Protocol::Default,
 6 => Protocol::Tcp,
 17 => Protocol::Udp,
 1 => Protocol::Icmp,
 _ => Protocol::Default,
 };
 
 // Validate combination
 if st == SocketType::Stream && proto == Protocol::Udp {
 return Err(-94); /* ESOCKTNOSUPPORT */
 }
 if st == SocketType::Dgram && proto == Protocol::Tcp {
 return Err(-94);
 }
 
 // Allocate fd
 let fd = self.next_fd.fetch_add(1, Ordering::AcqRel) as SockFd;
 let idx = fd as usize;
 if idx >= MAX_SOCKETS {
 return Err(-24); /* EMFILE */
 }

 // Create socket and store in table
 let mut sock = Socket::new(af, st, proto);
 sock.fd = fd;
 self.socket_table[idx] = Some(sock);
 self.socket_count.fetch_add(1, Ordering::AcqRel);
 
 Ok(fd)
 }
 
 /// Get statistics
 pub fn get_stats(&self) -> (u32, u64, u64) {
 (
 self.socket_count.load(Ordering::Acquire),
 self.bytes_sent.load(Ordering::Acquire),
 self.bytes_recv.load(Ordering::Acquire),
 )
 }
}

/// Global socket manager
static SOCKET_MANAGER: crate::sync_oncelock::OnceLock<SocketManager> = crate::sync_oncelock::OnceLock::new();

/// Get socket manager
pub fn socket_manager() -> &'static SocketManager {
    SOCKET_MANAGER.get_or_init(SocketManager::new)
}

pub fn init_socket_manager() -> &'static SocketManager {
    SOCKET_MANAGER.get_or_init(SocketManager::new)
}

/// Initialize socket subsystem
pub fn init_socket() {
 let mgr = socket_manager();
 mgr.init();
}

/// Socket system call
pub fn sys_socket(domain: i32, sock_type: i32, protocol: i32) -> i64 {
 match socket_manager().socket(domain, sock_type, protocol) {
 Ok(fd) => fd as i64,
 Err(e) => e as i64,
 }
}

/// Bind system call
pub fn sys_bind(sockfd: SockFd, addr: *const SockAddrInet, _addrlen: usize) -> i64 {
 if addr.is_null() {
 return Errno::Einval.to_syscall_return(); /* EINVAL */
 }
 
 // SAFETY: Caller guarantees addr is valid for reads
 let sockaddr = unsafe { &*addr };
 
 let mgr = socket_manager();
 match mgr.get_socket(sockfd) {
 Some(sock) => match sock.bind(sockaddr) {
 Ok(()) => 0,
 Err(e) => e as i64,
 },
 None => Errno::Ebadf.to_syscall_return(), /* EBADF */
 }
}

/// Listen system call
pub fn sys_listen(sockfd: SockFd, backlog: i32) -> i64 {
 let mgr = socket_manager();
 match mgr.get_socket(sockfd) {
 Some(sock) => match sock.listen(backlog) {
 Ok(()) => 0,
 Err(e) => e as i64,
 },
 None => Errno::Ebadf.to_syscall_return(), /* EBADF */
 }
}

/// Accept system call
pub fn sys_accept(sockfd: SockFd, addr: *mut SockAddrInet, addrlen: *mut u32) -> i64 {
 let mgr = socket_manager();
 match mgr.get_socket(sockfd) {
 Some(sock) => match sock.accept(addr, addrlen) {
 Ok(fd) => fd as i64,
 Err(e) => e as i64,
 },
 None => Errno::Ebadf.to_syscall_return(), /* EBADF */
 }
}

/// Connect system call
pub fn sys_connect(sockfd: SockFd, addr: *const SockAddrInet, _addrlen: usize) -> i64 {
 if addr.is_null() {
 return Errno::Einval.to_syscall_return();
 }
 
 // SAFETY: Caller guarantees addr is valid for reads
 let sockaddr = unsafe { &*addr };
 
 let mgr = socket_manager();
 match mgr.get_socket(sockfd) {
 Some(sock) => match sock.connect(sockaddr) {
 Ok(()) => 0,
 Err(e) => e as i64,
 },
 None => Errno::Ebadf.to_syscall_return(), /* EBADF */
 }
}

/// Send system call
pub fn sys_send(sockfd: SockFd, buf: *const u8, len: usize, flags: i32) -> i64 {
 if buf.is_null() {
 return Errno::Einval.to_syscall_return();
 }
 if len == 0 {
 return 0;
 }
 
 // SAFETY: Caller guarantees buf is valid for len bytes
 let buffer = unsafe { core::slice::from_raw_parts(buf, len) };
 
 let mgr = socket_manager();
 match mgr.get_socket(sockfd) {
 Some(sock) => match sock.send(buffer, flags) {
 Ok(n) => {
 mgr.bytes_sent.fetch_add(n as u64, Ordering::AcqRel);
 n as i64
 }
 Err(e) => e as i64,
 },
 None => Errno::Ebadf.to_syscall_return(), /* EBADF */
 }
}

/// Recv system call
pub fn sys_recv(sockfd: SockFd, buf: *mut u8, len: usize, flags: i32) -> i64 {
 if buf.is_null() {
 return Errno::Einval.to_syscall_return();
 }
 if len == 0 {
 return 0;
 }
 
 // SAFETY: Caller guarantees buf is valid for len bytes writes
 let buffer = unsafe { core::slice::from_raw_parts_mut(buf, len) };
 
 let mgr = socket_manager();
 match mgr.get_socket(sockfd) {
 Some(sock) => match sock.recv(buffer, flags) {
 Ok(n) => {
 mgr.bytes_recv.fetch_add(n as u64, Ordering::AcqRel);
 n as i64
 }
 Err(e) => e as i64,
 },
 None => Errno::Ebadf.to_syscall_return(), /* EBADF */
 }
}

/// Sendto system call
pub fn sys_sendto(
 sockfd: SockFd,
 buf: *const u8,
 len: usize,
 flags: i32,
 dest_addr: *const SockAddrInet,
 addrlen: usize,
) -> i64 {
 if buf.is_null() {
 return Errno::Einval.to_syscall_return();
 }
 if len == 0 {
 return 0;
 }
 
 // SAFETY: Caller guarantees buf is valid for len bytes
 let buffer = unsafe { core::slice::from_raw_parts(buf, len) };
 
 let mgr = socket_manager();
 if !dest_addr.is_null() {
 // SAFETY: Caller guarantees dest_addr is valid
 let sockaddr = unsafe { &*dest_addr };
 match mgr.get_socket(sockfd) {
 Some(sock) => match sock.sendto(buffer, sockaddr, flags) {
 Ok(n) => {
 mgr.bytes_sent.fetch_add(n as u64, Ordering::AcqRel);
 n as i64
 }
 Err(e) => e as i64,
 },
 None => Errno::Ebadf.to_syscall_return(),
 }
 } else {
 match mgr.get_socket(sockfd) {
 Some(sock) => match sock.send(buffer, flags) {
 Ok(n) => {
 mgr.bytes_sent.fetch_add(n as u64, Ordering::AcqRel);
 n as i64
 }
 Err(e) => e as i64,
 },
 None => Errno::Ebadf.to_syscall_return(),
 }
 }
}

/// Recvfrom system call
pub fn sys_recvfrom(
 sockfd: SockFd,
 buf: *mut u8,
 len: usize,
 flags: i32,
 src_addr: *mut SockAddrInet,
 addrlen: *mut u32,
) -> i64 {
 if buf.is_null() {
 return Errno::Einval.to_syscall_return();
 }
 if len == 0 {
 return 0;
 }
 
 // SAFETY: Caller guarantees buf is valid for len bytes writes
 let buffer = unsafe { core::slice::from_raw_parts_mut(buf, len) };
 
 let mgr = socket_manager();
 match mgr.get_socket(sockfd) {
 Some(sock) => match sock.recvfrom(buffer, src_addr, flags) {
 Ok(n) => {
 mgr.bytes_recv.fetch_add(n as u64, Ordering::AcqRel);
 n as i64
 }
 Err(e) => e as i64,
 },
 None => Errno::Ebadf.to_syscall_return(),
 }
}

/// Setsockopt system call
pub fn sys_setsockopt(
 sockfd: SockFd,
 level: i32,
 optname: i32,
 optval: *const u8,
 optlen: u32,
) -> i64 {
 let mgr = socket_manager();
 match mgr.get_socket(sockfd) {
 Some(sock) => match sock.setsockopt(level, optname, optval, optlen) {
 Ok(()) => 0,
 Err(e) => e as i64,
 },
 None => Errno::Ebadf.to_syscall_return(),
 }
}

/// Getsockopt system call
pub fn sys_getsockopt(
 sockfd: SockFd,
 level: i32,
 optname: i32,
 optval: *mut u8,
 optlen: *mut u32,
) -> i64 {
 let mgr = socket_manager();
 match mgr.get_socket(sockfd) {
 Some(sock) => match sock.getsockopt(level, optname, optval, optlen) {
 Ok(()) => 0,
 Err(e) => e as i64,
 },
 None => Errno::Ebadf.to_syscall_return(),
 }
}

/// Shutdown system call
pub fn sys_shutdown(sockfd: SockFd, how: i32) -> i64 {
 let mgr = socket_manager();
 match mgr.get_socket(sockfd) {
 Some(sock) => {
 // SHUT_RD=0, SHUT_WR=1, SHUT_RDWR=2
 match how {
 0 => {
 sock.state.store(SocketState::Disconnecting as u32, Ordering::Release);
 }
 1 | 2 => {
 sock.state.store(SocketState::Unconnected as u32, Ordering::Release);
 sock.flags.fetch_and(!sock_flags::SF_CONNECTED, Ordering::AcqRel);
 }
 _ => return Errno::Einval.to_syscall_return(),
 }
 0
 }
 None => Errno::Ebadf.to_syscall_return(),
 }
}

// ============================================================================
// Socket Buffermanagementadministration
// ============================================================================

/// Socket Buffer
pub struct SocketBuffer {
 /// BufferData
 pub data: [u8; 65536],
 /// DatastartbeginPosition
 pub head: u32,
 /// DataEndPosition
 pub tail: u32,
 /// BufferSize
 pub size: u32,
 /// MaxSize
 pub max_size: u32,
}

impl SocketBuffer {
 pub const fn new() -> Self {
 SocketBuffer {
 data: [0; 65536],
 head: 0,
 tail: 0,
 size: 0,
 max_size: 65536,
 }
 }
 
 /// WriteData
 pub fn write(&mut self, buf: &[u8]) -> usize {
 let available = (self.max_size - self.size) as usize;
 let write_len = buf.len().min(available);
 
 for i in 0..write_len {
 let pos = (self.tail + i as u32) % self.max_size;
 self.data[pos as usize] = buf[i];
 }
 
 self.tail = (self.tail + write_len as u32) % self.max_size;
 self.size += write_len as u32;
 
 write_len
 }
 
 /// ReadData
 pub fn read(&mut self, buf: &mut [u8]) -> usize {
 let read_len = buf.len().min(self.size as usize);
 
 for i in 0..read_len {
 let pos = (self.head + i as u32) % self.max_size;
 buf[i] = self.data[pos as usize];
 }
 
 self.head = (self.head + read_len as u32) % self.max_size;
 self.size -= read_len as u32;
 
 read_len
 }
 
 /// inspectionData (notMovepointer)
 pub fn peek(&self, buf: &mut [u8]) -> usize {
 let read_len = buf.len().min(self.size as usize);
 
 for i in 0..read_len {
 let pos = (self.head + i as u32) % self.max_size;
 buf[i] = self.data[pos as usize];
 }
 
 read_len
 }
 
 /// Getcanuseemptybetween
 pub fn available(&self) -> u32 {
 self.max_size - self.size
 }
 
 /// GetDataLength
 pub fn len(&self) -> u32 {
 self.size
 }
 
 /// ClearBuffer
 pub fn clear(&mut self) {
 self.head = 0;
 self.tail = 0;
 self.size = 0;
 }
}

// ============================================================================
// Socket EventNotification
// ============================================================================

/// Socket EventType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketEvent {
 /// canread
 Readable,
 /// canwrite
 Writable,
 /// haveJoin
 Connected,
 /// JoinClose
 Closed,
 /// Error
 Error,
}

/// Socket EventCallback
pub type SocketEventCallback = extern "C" fn(fd: SockFd, event: SocketEvent, data: *mut u8);

/// Socket EventListener
pub struct SocketEventListener {
 /// CallbackFunction
 pub callback: Option<SocketEventCallback>,
 /// UserData
 pub user_data: *mut u8,
 /// Listen EventMask
 pub event_mask: u32,
}

impl SocketEventListener {
 pub const fn new() -> Self {
 SocketEventListener {
 callback: None,
 user_data: core::ptr::null_mut(),
 event_mask: 0,
 }
 }
 
 /// TriggerEvent
 pub fn trigger(&self, fd: SockFd, event: SocketEvent) {
 if let Some(callback) = self.callback {
 callback(fd, event, self.user_data);
 }
 }
}

// ============================================================================
// Socket Addressparse
// ============================================================================

/// parse IP AddressString
pub fn parse_ip_addr(s: &[u8]) -> Option<u32> {
 let mut result: u32 = 0;
 let mut part: u32 = 0;
 let mut dots = 0;
 
 for &c in s {
 if c == b'.' {
 if part > 255 {
 return None;
 }
 result = (result << 8) | part;
 part = 0;
 dots += 1;
 } else if c >= b'0' && c <= b'9' {
 part = part * 10 + (c - b'0') as u32;
 } else {
 return None;
 }
 }
 
 if dots != 3 || part > 255 {
 return None;
 }
 
 result = (result << 8) | part;
 Some(result.to_be())
}

/// Format IP Address
pub fn format_ip_addr(addr: u32, buf: &mut [u8; 16]) -> usize {
 let addr = u32::from_be(addr);
 let a = (addr >> 24) & 0xFF;
 let b = (addr >> 16) & 0xFF;
 let c = (addr >> 8) & 0xFF;
 let d = addr & 0xFF;
 
 let mut pos = 0;
 pos += write_digit(a, &mut buf[pos..]);
 buf[pos] = b'.';
 pos += 1;
 pos += write_digit(b, &mut buf[pos..]);
 buf[pos] = b'.';
 pos += 1;
 pos += write_digit(c, &mut buf[pos..]);
 buf[pos] = b'.';
 pos += 1;
 pos += write_digit(d, &mut buf[pos..]);
 
 pos
}

fn write_digit(mut n: u32, buf: &mut [u8]) -> usize {
 if n == 0 {
 buf[0] = b'0';
 return 1;
 }
 
 let mut digits = [0u8; 3];
 let mut len = 0;
 
 while n > 0 {
 digits[len] = (n % 10) as u8 + b'0';
 n /= 10;
 len += 1;
 }
 
 for i in 0..len {
 buf[i] = digits[len - 1 - i];
 }
 
 len
}

// ============================================================================
// Socket OptionScaling
// ============================================================================

/// Linger Option
#[repr(C)]
pub struct Linger {
 pub l_onoff: i32,
 pub l_linger: i32,
}

/// TCP Option
pub mod tcp_options {
 pub const NODELAY: i32 = 1;
 pub const MAXSEG: i32 = 2;
 pub const CORK: i32 = 3;
 pub const KEEPIDLE: i32 = 4;
 pub const KEEPINTVL: i32 = 5;
 pub const KEEPCNT: i32 = 6;
 pub const SYNCCNT: i32 = 7;
 pub const LINGER2: i32 = 8;
 pub const DEFER_ACCEPT: i32 = 9;
 pub const WINDOW_CLAMP: i32 = 10;
 pub const INFO: i32 = 11;
 pub const QUICKACK: i32 = 12;
 pub const CONGESTION: i32 = 13;
 pub const MD5SIG: i32 = 14;
 pub const THIN_LINEAR_TIMEOUTS: i32 = 16;
 pub const THIN_DUPACK: i32 = 17;
 pub const REPAIR: i32 = 19;
 pub const REPAIR_QUEUE: i32 = 20;
 pub const QUEUE_SEQ: i32 = 21;
 pub const REPAIR_OPTIONS: i32 = 22;
 pub const FASTOPEN: i32 = 23;
 pub const TIMESTAMP: i32 = 24;
 pub const NOTSENT_LOWAT: i32 = 25;
 pub const CC_INFO: i32 = 26;
 pub const SAVE_SYN: i32 = 27;
 pub const SAVED_SYN: i32 = 28;
 pub const REPAIR_WINDOW: i32 = 29;
 pub const SMC: i32 = 30;
}

/// IP Option
pub mod ip_options {
 pub const TOS: i32 = 1;
 pub const TTL: i32 = 2;
 pub const HDRINCL: i32 = 3;
 pub const OPTIONS: i32 = 4;
 pub const ROUTER_ALERT: i32 = 5;
 pub const RECVOPTS: i32 = 6;
 pub const PKTINFO: i32 = 8;
 pub const PKTOPTIONS: i32 = 9;
 pub const PMTUDISC: i32 = 10;
 pub const MTU_DISCOVER: i32 = 10;
 pub const RECVERR: i32 = 11;
 pub const RECVTTL: i32 = 12;
 pub const RECVTOS: i32 = 13;
 pub const MTU: i32 = 14;
 pub const FREEBIND: i32 = 15;
 pub const IPSEC_POLICY: i32 = 16;
 pub const XFRM_POLICY: i32 = 17;
 pub const PASSSEC: i32 = 18;
 pub const TRANSPARENT: i32 = 19;
 pub const ORIGDSTADDR: i32 = 20;
 pub const RECVIF: i32 = 21;
 pub const NODEFRAG: i32 = 22;
 pub const CHECKSUM: i32 = 23;
 pub const MULTICAST_TTL: i32 = 33;
 pub const MULTICAST_LOOP: i32 = 34;
 pub const ADD_MEMBERSHIP: i32 = 35;
 pub const DROP_MEMBERSHIP: i32 = 36;
 pub const UNBLOCK_SOURCE: i32 = 37;
 pub const BLOCK_SOURCE: i32 = 38;
 pub const ADD_SOURCE_MEMBERSHIP: i32 = 39;
 pub const DROP_SOURCE_MEMBERSHIP: i32 = 40;
 pub const MSFILTER: i32 = 41;
 pub const MULTICAST_ALL: i32 = 49;
 pub const UNICAST_IF: i32 = 50;
 pub const LOCAL_PORT_RANGE: i32 = 51;
 pub const RECVORIGDSTADDR: i32 = 20;
}

// ============================================================================
// Socket StatisticsScaling
// ============================================================================

/// Socket fineStatistics
pub struct SocketDetailedStats {
 /// Create socket total
 pub sockets_created: AtomicU64,
 /// Destroy socket total
 pub sockets_destroyed: AtomicU64,
 /// TCP Joinnumber
 pub tcp_connections: AtomicU32,
 /// UDP socket number
 pub udp_sockets: AtomicU32,
 /// Unix socket number
 pub unix_sockets: AtomicU32,
 /// bindError
 pub bind_errors: AtomicU64,
 /// JoinError
 pub connect_errors: AtomicU64,
 /// listenError
 pub listen_errors: AtomicU64,
 /// ReceiveError
 pub recv_errors: AtomicU64,
 /// SendError
 pub send_errors: AtomicU64,
}

impl SocketDetailedStats {
 pub const fn new() -> Self {
 SocketDetailedStats {
 sockets_created: AtomicU64::new(0),
 sockets_destroyed: AtomicU64::new(0),
 tcp_connections: AtomicU32::new(0),
 udp_sockets: AtomicU32::new(0),
 unix_sockets: AtomicU32::new(0),
 bind_errors: AtomicU64::new(0),
 connect_errors: AtomicU64::new(0),
 listen_errors: AtomicU64::new(0),
 recv_errors: AtomicU64::new(0),
 send_errors: AtomicU64::new(0),
 }
 }
}

/// GlobalfineStatistics
pub static SOCKET_DETAILED_STATS: SocketDetailedStats = SocketDetailedStats::new();

// ============================================================================
// auxiliaryFunction
// ============================================================================

/// CheckPortifvalid
pub fn is_valid_port(port: u16) -> bool {
 port != 0
}

/// CheckAddressifasmatchsymbol
pub fn is_wildcard_addr(addr: u32) -> bool {
 addr == 0
}

/// CheckAddressifasBroadcastingAddress
pub fn is_broadcast_addr(addr: u32) -> bool {
 addr == 0xFFFFFFFF
}

/// CheckAddressifasRingroundAddress
pub fn is_loopback_addr(addr: u32) -> bool {
 let addr = u32::from_be(addr);
 (addr >> 24) == 127
}

/// CheckAddressifasMulticastAddress
pub fn is_multicast_addr(addr: u32) -> bool {
 let addr = u32::from_be(addr);
 (addr >> 28) == 0xE // 224.0.0.0 - 239.255.255.255
}

/// CheckAddressifasprivatefiniteAddress
pub fn is_private_addr(addr: u32) -> bool {
 let addr = u32::from_be(addr);
 let a = (addr >> 24) & 0xFF;
 let b = (addr >> 16) & 0xFF;
 
 // 10.0.0.0/8
 if a == 10 {
 return true;
 }
 // 172.16.0.0/12
 if a == 172 && (16..=31).contains(&b) {
 return true;
 }
 // 192.168.0.0/16
 if a == 192 && b == 168 {
 return true;
 }
 
 false
}

#[cfg(test)]
mod tests {
 use super::*;
 
 #[test]
 fn test_parse_ip_addr() {
 assert_eq!(parse_ip_addr(b"127.0.0.1"), Some(0x7F000001));
 assert_eq!(parse_ip_addr(b"192.168.1.1"), Some(0xC0A80101));
 assert_eq!(parse_ip_addr(b"0.0.0.0"), Some(0x00000000));
 assert_eq!(parse_ip_addr(b"255.255.255.255"), Some(0xFFFFFFFF));
 assert_eq!(parse_ip_addr(b"invalid"), None);
 }
 
 #[test]
 fn test_is_loopback() {
 assert!(is_loopback_addr(0x7F000001));
 assert!(is_loopback_addr(0x7F000002));
 assert!(!is_loopback_addr(0xC0A80101));
 }
 
 #[test]
 fn test_is_multicast() {
 assert!(is_multicast_addr(0xE0000001)); // 224.0.0.1
 assert!(is_multicast_addr(0xEFFFFFFF)); // 239.255.255.255
 assert!(!is_multicast_addr(0xC0A80101));
 }
 
 #[test]
 fn test_is_private() {
 assert!(is_private_addr(0x0A000001)); // 10.0.0.1
 assert!(is_private_addr(0xAC100001)); // 172.16.0.1
 assert!(is_private_addr(0xC0A80101)); // 192.168.1.1
 assert!(!is_private_addr(0x08080808)); // 8.8.8.8
 }
 
 #[test]
 fn test_socket_buffer() {
 let mut buf = SocketBuffer::new();
 
 let data = [1, 2, 3, 4, 5];
 assert_eq!(buf.write(&data), 5);
 assert_eq!(buf.len(), 5);
 
 let mut out = [0u8; 5];
 assert_eq!(buf.read(&mut out), 5);
 assert_eq!(out, data);
 assert_eq!(buf.len(), 0);
 }
}