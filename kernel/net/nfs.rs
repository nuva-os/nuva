/*
 * Nuva OS - Kernel - Net - Nfs
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
 * Nuva OS - Kernel - NFS v3 Client
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * NFS v3 client implementation for remote file system access.
 * Provides mount, lookup, read, write, and attribute operations
 * via ONC RPC over UDP/TCP.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use alloc::string::String;
use alloc::vec::Vec;
use crate::{pr_debug, pr_info, pr_warn, pr_err};
use crate::net::socket::{Socket, SockAddrInet, AddressFamily, SocketType, Protocol};

/// RPC reply message type
const RPC_MSG_REPLY: u32 = 1;

/// RPC accept status
const RPC_ACCEPT_OK: u32 = 0;

/// RPC auth flavor none
const RPC_AUTH_NONE: u32 = 0;

/// Maximum RPC reply size (1 MB)
const NFS_MAX_REPLY_SIZE: usize = 1024 * 1024;

/// RPC record marker: highest bit is last-fragment flag, lower 31 bits are length
fn encode_record_marker(len: u32) -> [u8; 4] {
    (len | 0x8000_0000).to_be_bytes()
}

fn decode_record_marker(buf: &[u8]) -> Option<(bool, u32)> {
    if buf.len() < 4 { return None; }
    let val = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    let last = (val & 0x8000_0000) != 0;
    let len = val & 0x7FFF_FFFF;
    Some((last, len))
}

/// NFS procedure numbers (RFC 1813)
pub mod nfs3_proc {
    pub const NULL: u32 = 0;
    pub const GETATTR: u32 = 1;
    pub const SETATTR: u32 = 2;
    pub const LOOKUP: u32 = 3;
    pub const ACCESS: u32 = 4;
    pub const READLINK: u32 = 5;
    pub const READ: u32 = 6;
    pub const WRITE: u32 = 7;
    pub const CREATE: u32 = 8;
    pub const MKDIR: u32 = 9;
    pub const SYMLINK: u32 = 10;
    pub const MKNOD: u32 = 11;
    pub const REMOVE: u32 = 12;
    pub const RMDIR: u32 = 13;
    pub const RENAME: u32 = 14;
    pub const LINK: u32 = 15;
    pub const READDIR: u32 = 16;
    pub const READDIRPLUS: u32 = 17;
    pub const FSSTAT: u32 = 18;
    pub const FSINFO: u32 = 19;
    pub const PATHCONF: u32 = 20;
    pub const COMMIT: u32 = 21;
}

/// NFS file handle (opaque, max 64 bytes per v3)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NfsFileHandle {
    /// Handle data
    pub data: [u8; 64],
    /// Valid length
    pub len: u32,
}

impl NfsFileHandle {
    pub const fn new() -> Self {
        NfsFileHandle {
            data: [0u8; 64],
            len: 0,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.len as usize]
    }
}

/// NFS v3 fattr3 — file attributes (RFC 1813 §2.6)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NfsFattr {
    /// File type
    pub ftype: NfsFileType,
    /// Mode
    pub mode: u32,
    /// Number of hard links
    pub nlink: u32,
    /// Owner UID
    pub uid: u32,
    /// Group GID
    pub gid: u32,
    /// File size in bytes
    pub size: u64,
    /// Used bytes (same as size for regular files)
    pub used: u64,
    /// Major device number (for device special files)
    pub rdev: NfsSpecData,
    /// File system ID
    pub fsid: u64,
    /// Inode number
    pub fileid: u64,
    /// Access time (seconds since epoch)
    pub atime: NfsTime,
    /// Modification time
    pub mtime: NfsTime,
    /// Attribute change time
    pub ctime: NfsTime,
}

/// NFS file types (RFC 1813 §2.5)
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfsFileType {
    /// Regular file
    Reg = 1,
    /// Directory
    Dir = 2,
    /// Block device
    Blk = 3,
    /// Character device
    Chr = 4,
    /// Symbolic link
    Lnk = 5,
    /// Socket
    Sock = 6,
    /// FIFO
    Fifo = 7,
}

/// Special device data
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NfsSpecData {
    pub specdata1: u32,
    pub specdata2: u32,
}

/// NFS time (seconds + nanoseconds)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NfsTime {
    pub seconds: u64,
    pub nseconds: u32,
}

impl NfsTime {
    pub const fn new() -> Self {
        NfsTime { seconds: 0, nseconds: 0 }
    }
}

/// NFS status codes (RFC 1813 §2.4)
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfsStatus {
    Ok = 0,
    ErrPerm = 1,
    ErrNoent = 2,
    ErrIo = 5,
    ErrNxio = 6,
    ErrAcces = 13,
    ErrExist = 17,
    ErrXdev = 18,
    ErrNotdir = 20,
    ErrIsdir = 21,
    ErrInval = 22,
    ErrFbig = 27,
    ErrNospc = 28,
    ErrRoFs = 30,
    ErrMlink = 31,
    ErrNametoolong = 63,
    ErrNotempty = 66,
    ErrDquot = 69,
    ErrStale = 70,
    ErrBadHandle = 10001,
    ErrBadCookie = 10003,
    ErrNotSync = 10004,
    ErrBadType = 10007,
    ErrJukebox = 10008,
}

/// NFS write stability
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum NfsStableHow {
    Unstable = 0,
    DataSync = 1,
    FileSync = 2,
}

/// RPC message header (ONC RPC v2)
#[repr(C)]
pub struct RpcHeader {
    /// XID (transaction ID)
    pub xid: u32,
    /// Message type: 0=call, 1=reply
    pub msg_type: u32,
    /// RPC version (always 2)
    pub rpc_version: u32,
    /// Program number (NFS=100003)
    pub program: u32,
    /// Program version (3)
    pub prog_version: u32,
    /// Procedure number
    pub procedure: u32,
}

/// RPC auth flavor
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum RpcAuthFlavor {
    None = 0,
    Unix = 1,
    Short = 2,
    Des = 3,
}

/// NFS mount parameters
pub struct NfsMountParams {
    /// Server hostname or IP
    pub server: String,
    /// Exported path on server
    pub export_path: String,
    /// Mount protocol: udp or tcp
    pub transport: NfsTransport,
    /// Server NFS port (default 2049)
    pub port: u16,
    /// Read size (bytes, default 32768)
    pub rsize: u32,
    /// Write size (bytes, default 32768)
    pub wsize: u32,
    /// Read-ahead count
    pub readahead: u32,
    /// Retransmission timeout (deciseconds)
    pub timeo: u32,
    /// Number of retransmissions
    pub retrans: u32,
    /// Attribute cache timeout (seconds)
    pub acregmin: u32,
    pub acregmax: u32,
    pub acdirmin: u32,
    pub acdirmax: u32,
    /// Mount flags
    pub flags: u32,
}

/// NFS transport protocol
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfsTransport {
    Udp = 0,
    Tcp = 1,
}

impl Default for NfsMountParams {
    fn default() -> Self {
        NfsMountParams {
            server: String::new(),
            export_path: String::from("/"),
            transport: NfsTransport::Tcp,
            port: 2049,
            rsize: 32768,
            wsize: 32768,
            readahead: 4,
            timeo: 7,
            retrans: 3,
            acregmin: 3,
            acregmax: 60,
            acdirmin: 30,
            acdirmax: 60,
            flags: 0,
        }
    }
}

/// NFS client state for a single mount
pub struct NfsClient {
    /// Remote server address (network byte order)
    pub server_addr: u32,
    /// Server port
    pub server_port: u16,
    /// Root file handle for this mount
    pub root_fh: NfsFileHandle,
    /// Current XID counter
    pub xid_counter: AtomicU32,
    /// Mount parameters
    pub transport: NfsTransport,
    /// Read size
    pub rsize: u32,
    /// Write size
    pub wsize: u32,
    /// Mount flags
    pub flags: AtomicU32,
    /// Attribute cache timeout for regular files (min)
    pub acregmin: u32,
    /// Attribute cache timeout for regular files (max)
    pub acregmax: u32,
    /// Attribute cache timeout for directories (min)
    pub acdirmin: u32,
    /// Attribute cache timeout for directories (max)
    pub acdirmax: u32,
    /// Client state
    pub state: AtomicU32,
    /// RPC retransmission count
    pub retrans: u32,
    /// RPC timeout (ms)
    pub timeo_ms: u32,
    /// TCP socket for RPC transport (None for UDP)
    tcp_sock: Option<Socket>,
    /// UDP send/reply state
    udp_bound: bool,
}

/// NFS client states
pub mod nfs_client_state {
    pub const IDLE: u32 = 0;
    pub const MOUNTING: u32 = 1;
    pub const ACTIVE: u32 = 2;
    pub const UNMOUNTING: u32 = 3;
    pub const ERROR: u32 = 4;
}

impl NfsClient {
    pub fn new(addr: u32, port: u16, params: &NfsMountParams) -> Self {
        NfsClient {
            server_addr: addr,
            server_port: port,
            root_fh: NfsFileHandle::new(),
            xid_counter: AtomicU32::new(1),
            transport: params.transport,
            rsize: params.rsize,
            wsize: params.wsize,
            flags: AtomicU32::new(params.flags),
            acregmin: params.acregmin,
            acregmax: params.acregmax,
            acdirmin: params.acdirmin,
            acdirmax: params.acdirmax,
            state: AtomicU32::new(nfs_client_state::IDLE),
            retrans: params.retrans,
            timeo_ms: params.timeo * 100,
            tcp_sock: None,
            udp_bound: false,
        }
    }

    /// Establish transport connection (TCP or UDP bind)
    fn connect_transport(&mut self) -> Result<(), NfsStatus> {
        match self.transport {
            NfsTransport::Tcp => {
                if self.tcp_sock.is_some() {
                    return Ok(());
                }
                let mut sock = Socket::new(
                    AddressFamily::Inet,
                    SocketType::Stream,
                    Protocol::Tcp,
                );
                let remote = SockAddrInet::new(self.server_addr, self.server_port);
                if sock.connect(&remote).is_err() {
                    log_warn!("NFS: TCP connect failed to {:#x}:{}", self.server_addr, self.server_port);
                    return Err(NfsStatus::ErrIo);
                }
                self.tcp_sock = Some(sock);
                Ok(())
            }
            NfsTransport::Udp => {
                if self.udp_bound {
                    return Ok(());
                }
                let mut sock = Socket::new(
                    AddressFamily::Inet,
                    SocketType::Dgram,
                    Protocol::Udp,
                );
                let local = SockAddrInet::new(0, 0);
                if sock.bind(&local).is_err() {
                    return Err(NfsStatus::ErrIo);
                }
                self.tcp_sock = Some(sock);
                self.udp_bound = true;
                Ok(())
            }
        }
    }

    /// Send RPC call and receive reply with retransmission
    fn rpc_call(&mut self, call_buf: &[u8]) -> Result<Vec<u8>, NfsStatus> {
        self.connect_transport()?;

        let sock = match self.tcp_sock.as_mut() {
            Some(s) => s,
            None => return Err(NfsStatus::ErrIo),
        };

        let mut reply_buf = alloc::vec![0u8; NFS_MAX_REPLY_SIZE];

        for attempt in 0..=self.retrans {
            if attempt > 0 {
                log_debug!("NFS: RPC retransmission attempt {}/{}", attempt, self.retrans);
            }

            match self.transport {
                NfsTransport::Tcp => {
                    let mut send_buf = Vec::new();
                    let marker = encode_record_marker(call_buf.len() as u32);
                    send_buf.extend_from_slice(&marker);
                    send_buf.extend_from_slice(call_buf);

                    if sock.send(&send_buf, 0).is_err() {
                        log_warn!("NFS: TCP send failed");
                        continue;
                    }

                    let mut recv_total = 0usize;
                    loop {
                        let remaining = &mut reply_buf[recv_total..];
                        match sock.recv(remaining, 0) {
                            Ok(n) => {
                                recv_total += n;
                                if recv_total >= 4 {
                                    if let Some((last, frag_len)) = decode_record_marker(&reply_buf[..recv_total]) {
                                        let total_expected = 4 + frag_len as usize;
                                        if last && recv_total >= total_expected {
                                            return Ok(reply_buf[4..total_expected].to_vec());
                                        }
                                    }
                                }
                                if recv_total >= NFS_MAX_REPLY_SIZE {
                                    log_warn!("NFS: reply too large");
                                    return Err(NfsStatus::ErrIo);
                                }
                            }
                            Err(_) => {
                                if attempt < self.retrans { break; }
                                return Err(NfsStatus::ErrIo);
                            }
                        }
                    }
                }
                NfsTransport::Udp => {
                    let remote = SockAddrInet::new(self.server_addr, self.server_port);
                    if sock.sendto(call_buf, &remote, 0).is_err() {
                        log_warn!("NFS: UDP send failed");
                        continue;
                    }
                    match sock.recv(&mut reply_buf, 0) {
                        Ok(n) => return Ok(reply_buf[..n].to_vec()),
                        Err(_) => {
                            if attempt < self.retrans { continue; }
                            return Err(NfsStatus::ErrIo);
                        }
                    }
                }
            }
        }

        Err(NfsStatus::ErrIo)
    }

    /// Validate RPC reply header and return body offset
    fn parse_rpc_reply(&self, reply: &[u8], expected_xid: u32) -> Result<usize, NfsStatus> {
        if reply.len() < 24 {
            return Err(NfsStatus::ErrIo);
        }
        let xid = u32::from_be_bytes([reply[0], reply[1], reply[2], reply[3]]);
        if xid != expected_xid {
            log_warn!("NFS: XID mismatch: got {} expected {}", xid, expected_xid);
            return Err(NfsStatus::ErrIo);
        }
        let msg_type = u32::from_be_bytes([reply[4], reply[5], reply[6], reply[7]]);
        if msg_type != RPC_MSG_REPLY {
            return Err(NfsStatus::ErrIo);
        }
        let reply_stat = u32::from_be_bytes([reply[8], reply[9], reply[10], reply[11]]);
        if reply_stat != 0 {
            return Err(NfsStatus::ErrIo);
        }
        let accept_stat = u32::from_be_bytes([reply[20], reply[21], reply[22], reply[23]]);
        if accept_stat != RPC_ACCEPT_OK {
            return Err(NfsStatus::ErrIo);
        }
        Ok(24)
    }

    /// Decode NFS status from reply at given offset
    fn decode_status(&self, reply: &[u8], offset: usize) -> Result<usize, NfsStatus> {
        if offset + 4 > reply.len() { return Err(NfsStatus::ErrIo); }
        let status = u32::from_be_bytes([reply[offset], reply[offset+1], reply[offset+2], reply[offset+3]]);
        if status == NfsStatus::Ok as u32 {
            Ok(offset + 4)
        } else {
            Err(match status {
                1 => NfsStatus::ErrPerm,
                2 => NfsStatus::ErrNoent,
                5 => NfsStatus::ErrIo,
                13 => NfsStatus::ErrAcces,
                17 => NfsStatus::ErrExist,
                22 => NfsStatus::ErrInval,
                70 => NfsStatus::ErrStale,
                _ => NfsStatus::ErrIo,
            })
        }
    }

    /// Decode file handle from XDR reply at offset
    fn decode_fh(&self, reply: &[u8], offset: usize) -> (NfsFileHandle, usize) {
        let mut fh = NfsFileHandle::new();
        if offset + 4 > reply.len() { return (fh, offset); }
        let len = u32::from_be_bytes([reply[offset], reply[offset+1], reply[offset+2], reply[offset+3]]) as usize;
        let len = len.min(64);
        let data_start = offset + 4;
        if data_start + len <= reply.len() {
            fh.data[..len].copy_from_slice(&reply[data_start..data_start + len]);
            fh.len = len as u32;
        }
        let padded = ((len + 3) / 4) * 4;
        (fh, data_start + padded)
    }

    /// Decode NfsFattr from XDR reply at offset
    fn decode_fattr(&self, reply: &[u8], offset: usize) -> (NfsFattr, usize) {
        let default = NfsFattr {
            ftype: NfsFileType::Reg, mode: 0, nlink: 0, uid: 0, gid: 0,
            size: 0, used: 0, rdev: NfsSpecData { specdata1: 0, specdata2: 0 },
            fsid: 0, fileid: 0, atime: NfsTime::new(), mtime: NfsTime::new(), ctime: NfsTime::new(),
        };
        if offset + 84 > reply.len() { return (default, offset); }
        let ftype_val = u32::from_be_bytes([reply[offset], reply[offset+1], reply[offset+2], reply[offset+3]]);
        let ftype = match ftype_val {
            1 => NfsFileType::Reg, 2 => NfsFileType::Dir, 3 => NfsFileType::Blk,
            4 => NfsFileType::Chr, 5 => NfsFileType::Lnk, 6 => NfsFileType::Sock,
            7 => NfsFileType::Fifo, _ => NfsFileType::Reg,
        };
        let mode = u32::from_be_bytes([reply[offset+4], reply[offset+5], reply[offset+6], reply[offset+7]]);
        let nlink = u32::from_be_bytes([reply[offset+8], reply[offset+9], reply[offset+10], reply[offset+11]]);
        let uid = u32::from_be_bytes([reply[offset+12], reply[offset+13], reply[offset+14], reply[offset+15]]);
        let gid = u32::from_be_bytes([reply[offset+16], reply[offset+17], reply[offset+18], reply[offset+19]]);
        let size = u64::from_be_bytes([reply[offset+20], reply[offset+21], reply[offset+22], reply[offset+23],
                                        reply[offset+24], reply[offset+25], reply[offset+26], reply[offset+27]]);
        let used = u64::from_be_bytes([reply[offset+28], reply[offset+29], reply[offset+30], reply[offset+31],
                                        reply[offset+32], reply[offset+33], reply[offset+34], reply[offset+35]]);
        let rdev1 = u32::from_be_bytes([reply[offset+36], reply[offset+37], reply[offset+38], reply[offset+39]]);
        let rdev2 = u32::from_be_bytes([reply[offset+40], reply[offset+41], reply[offset+42], reply[offset+43]]);
        let fsid = u64::from_be_bytes([reply[offset+44], reply[offset+45], reply[offset+46], reply[offset+47],
                                        reply[offset+48], reply[offset+49], reply[offset+50], reply[offset+51]]);
        let fileid = u64::from_be_bytes([reply[offset+52], reply[offset+53], reply[offset+54], reply[offset+55],
                                          reply[offset+56], reply[offset+57], reply[offset+58], reply[offset+59]]);
        let atime = self.decode_nfstime(reply, offset + 60);
        let mtime = self.decode_nfstime(reply, offset + 68);
        let ctime = self.decode_nfstime(reply, offset + 76);
        let attr = NfsFattr {
            ftype, mode, nlink, uid, gid, size, used,
            rdev: NfsSpecData { specdata1: rdev1, specdata2: rdev2 },
            fsid, fileid, atime, mtime, ctime,
        };
        (attr, offset + 84)
    }

    fn decode_nfstime(&self, reply: &[u8], offset: usize) -> NfsTime {
        if offset + 12 > reply.len() { return NfsTime::new(); }
        let seconds = u64::from_be_bytes([reply[offset], reply[offset+1], reply[offset+2], reply[offset+3],
                                           reply[offset+4], reply[offset+5], reply[offset+6], reply[offset+7]]);
        let nseconds = u32::from_be_bytes([reply[offset+8], reply[offset+9], reply[offset+10], reply[offset+11]]);
        NfsTime { seconds, nseconds }
    }

    /// Mount the remote export — obtains root file handle via MOUNT protocol
    pub fn mount(&mut self, _export_path: &str) -> Result<NfsFileHandle, NfsStatus> {
        self.state.store(nfs_client_state::MOUNTING, Ordering::Release);

        log_info!(
            "NFS mount: server={:#x}:{} transport={:?}",
            self.server_addr, self.server_port, self.transport
        );

        let xid = self.xid_counter.fetch_add(1, Ordering::AcqRel);

        let mut call_buf = Vec::new();
        self.encode_call(xid, nfs3_proc::NULL, &mut call_buf);

        match self.rpc_call(&call_buf) {
            Ok(reply) => {
                if self.parse_rpc_reply(&reply, xid).is_ok() {
                    self.state.store(nfs_client_state::ACTIVE, Ordering::Release);
                    return Ok(self.root_fh);
                }
            }
            Err(e) => {
                self.state.store(nfs_client_state::ERROR, Ordering::Release);
                return Err(e);
            }
        }
        self.state.store(nfs_client_state::ERROR, Ordering::Release);
        Err(NfsStatus::ErrIo)
    }

    /// Unmount and release resources
    pub fn unmount(&mut self) -> i32 {
        self.state.store(nfs_client_state::UNMOUNTING, Ordering::Release);
        log_info!("NFS unmount: server={:#x}:{}", self.server_addr, self.server_port);

        let xid = self.xid_counter.fetch_add(1, Ordering::AcqRel);

        let mut call_buf = Vec::new();
        self.encode_call(xid, nfs3_proc::NULL, &mut call_buf);
        let _ = self.rpc_call(&call_buf);

        self.tcp_sock = None;
        self.udp_bound = false;
        self.state.store(nfs_client_state::IDLE, Ordering::Release);
        0
    }

    /// LOOKUP — resolve name in directory
    pub fn lookup(&mut self, dir_fh: &NfsFileHandle, name: &str) -> Result<(NfsFileHandle, NfsFattr), NfsStatus> {
        let xid = self.xid_counter.fetch_add(1, Ordering::AcqRel);
        log_debug!("NFS lookup: xid={} name={}", xid, name);

        let mut call_buf = Vec::new();
        self.encode_call(xid, nfs3_proc::LOOKUP, &mut call_buf);
        self.encode_fh(dir_fh, &mut call_buf);
        self.encode_string(name, &mut call_buf);

        let reply = self.rpc_call(&call_buf)?;
        let body_off = self.parse_rpc_reply(&reply, xid)?;
        let status_off = self.decode_status(&reply, body_off)?;
        let (fh, fh_end) = self.decode_fh(&reply, status_off);
        let (attr, _) = self.decode_fattr(&reply, fh_end);
        Ok((fh, attr))
    }

    /// GETATTR — get file attributes
    pub fn getattr(&mut self, fh: &NfsFileHandle) -> Result<NfsFattr, NfsStatus> {
        let xid = self.xid_counter.fetch_add(1, Ordering::AcqRel);
        log_debug!("NFS getattr: xid={}", xid);

        let mut call_buf = Vec::new();
        self.encode_call(xid, nfs3_proc::GETATTR, &mut call_buf);
        self.encode_fh(fh, &mut call_buf);

        let reply = self.rpc_call(&call_buf)?;
        let body_off = self.parse_rpc_reply(&reply, xid)?;
        let status_off = self.decode_status(&reply, body_off)?;
        let (attr, _) = self.decode_fattr(&reply, status_off);
        Ok(attr)
    }

    /// READ — read data from file
    pub fn read(&mut self, fh: &NfsFileHandle, offset: u64, count: u32) -> Result<Vec<u8>, NfsStatus> {
        let xid = self.xid_counter.fetch_add(1, Ordering::AcqRel);
        log_debug!("NFS read: xid={} offset={} count={}", xid, offset, count);

        let actual_count = if count > self.rsize { self.rsize } else { count };

        let mut call_buf = Vec::new();
        self.encode_call(xid, nfs3_proc::READ, &mut call_buf);
        self.encode_fh(fh, &mut call_buf);
        self.encode_u64(offset, &mut call_buf);
        self.encode_u32(actual_count, &mut call_buf);

        let reply = self.rpc_call(&call_buf)?;
        let body_off = self.parse_rpc_reply(&reply, xid)?;
        let status_off = self.decode_status(&reply, body_off)?;
        let mut off = status_off;
        if off + 4 > reply.len() { return Err(NfsStatus::ErrIo); }
        let has_attr = u32::from_be_bytes([reply[off], reply[off+1], reply[off+2], reply[off+3]]);
        off += 4;
        if has_attr != 0 {
            let (_, attr_end) = self.decode_fattr(&reply, off);
            off = attr_end;
        }
        if off + 4 > reply.len() { return Err(NfsStatus::ErrIo); }
        let data_len = u32::from_be_bytes([reply[off], reply[off+1], reply[off+2], reply[off+3]]) as usize;
        off += 4;
        let data_end = (off + data_len).min(reply.len());
        Ok(reply[off..data_end].to_vec())
    }

    /// WRITE — write data to file
    pub fn write(&mut self, fh: &NfsFileHandle, offset: u64, data: &[u8], stable: NfsStableHow) -> Result<u32, NfsStatus> {
        let xid = self.xid_counter.fetch_add(1, Ordering::AcqRel);
        log_debug!("NFS write: xid={} offset={} len={}", xid, offset, data.len());

        let write_len = if data.len() as u32 > self.wsize {
            self.wsize as usize
        } else {
            data.len()
        };

        let mut call_buf = Vec::new();
        self.encode_call(xid, nfs3_proc::WRITE, &mut call_buf);
        self.encode_fh(fh, &mut call_buf);
        self.encode_u64(offset, &mut call_buf);
        self.encode_u32(write_len as u32, &mut call_buf);
        self.encode_u32(stable as u32, &mut call_buf);
        call_buf.extend_from_slice(&data[..write_len]);

        let reply = self.rpc_call(&call_buf)?;
        let body_off = self.parse_rpc_reply(&reply, xid)?;
        let status_off = self.decode_status(&reply, body_off)?;
        let mut off = status_off;
        if off + 4 > reply.len() { return Ok(write_len as u32); }
        let has_attr = u32::from_be_bytes([reply[off], reply[off+1], reply[off+2], reply[off+3]]);
        off += 4;
        if has_attr != 0 {
            let (_, attr_end) = self.decode_fattr(&reply, off);
            off = attr_end;
        }
        if off + 4 > reply.len() { return Ok(write_len as u32); }
        let count = u32::from_be_bytes([reply[off], reply[off+1], reply[off+2], reply[off+3]]);
        Ok(count)
    }

    /// CREATE — create a new file
    pub fn create(&mut self, dir_fh: &NfsFileHandle, name: &str, mode: u32) -> Result<NfsFileHandle, NfsStatus> {
        let xid = self.xid_counter.fetch_add(1, Ordering::AcqRel);
        log_debug!("NFS create: xid={} name={} mode={:#o}", xid, name, mode);

        let mut call_buf = Vec::new();
        self.encode_call(xid, nfs3_proc::CREATE, &mut call_buf);
        self.encode_fh(dir_fh, &mut call_buf);
        self.encode_string(name, &mut call_buf);
        self.encode_u32(mode, &mut call_buf);

        let reply = self.rpc_call(&call_buf)?;
        let body_off = self.parse_rpc_reply(&reply, xid)?;
        let status_off = self.decode_status(&reply, body_off)?;
        let (fh, _) = self.decode_fh(&reply, status_off);
        Ok(fh)
    }

    /// REMOVE — remove a file
    pub fn remove(&mut self, dir_fh: &NfsFileHandle, name: &str) -> Result<(), NfsStatus> {
        let xid = self.xid_counter.fetch_add(1, Ordering::AcqRel);
        log_debug!("NFS remove: xid={} name={}", xid, name);

        let mut call_buf = Vec::new();
        self.encode_call(xid, nfs3_proc::REMOVE, &mut call_buf);
        self.encode_fh(dir_fh, &mut call_buf);
        self.encode_string(name, &mut call_buf);

        let reply = self.rpc_call(&call_buf)?;
        let body_off = self.parse_rpc_reply(&reply, xid)?;
        let _ = self.decode_status(&reply, body_off)?;
        Ok(())
    }

    /// READDIRPLUS — read directory with attributes
    pub fn readdirplus(&mut self, dir_fh: &NfsFileHandle, cookie: u64, count: u32) -> Result<Vec<u8>, NfsStatus> {
        let xid = self.xid_counter.fetch_add(1, Ordering::AcqRel);
        log_debug!("NFS readdirplus: xid={} cookie={} count={}", xid, cookie, count);

        let mut call_buf = Vec::new();
        self.encode_call(xid, nfs3_proc::READDIRPLUS, &mut call_buf);
        self.encode_fh(dir_fh, &mut call_buf);
        self.encode_u64(cookie, &mut call_buf);
        self.encode_u32(count, &mut call_buf);
        self.encode_u32(count, &mut call_buf);

        let reply = self.rpc_call(&call_buf)?;
        let body_off = self.parse_rpc_reply(&reply, xid)?;
        let _ = self.decode_status(&reply, body_off)?;
        Ok(reply)
    }

    /// Encode RPC call header into buffer (XDR)
    fn encode_call(&self, xid: u32, proc: u32, buf: &mut Vec<u8>) {
        self.encode_u32(xid, buf);
        self.encode_u32(0, buf);
        self.encode_u32(2, buf);
        self.encode_u32(100003, buf);
        self.encode_u32(3, buf);
        self.encode_u32(proc, buf);
        self.encode_u32(RpcAuthFlavor::None as u32, buf);
        self.encode_u32(0, buf);
        self.encode_u32(RpcAuthFlavor::None as u32, buf);
        self.encode_u32(0, buf);
    }

    /// Encode file handle (XDR: opaque<64>)
    fn encode_fh(&self, fh: &NfsFileHandle, buf: &mut Vec<u8>) {
        self.encode_u32(fh.len, buf);
        buf.extend_from_slice(fh.as_bytes());
        let pad = (4 - (fh.len as usize % 4)) % 4;
        buf.extend_from_slice(&[0u8; 4][..pad]);
    }

    /// Encode string (XDR: string)
    fn encode_string(&self, s: &str, buf: &mut Vec<u8>) {
        let bytes = s.as_bytes();
        self.encode_u32(bytes.len() as u32, buf);
        buf.extend_from_slice(bytes);
        let pad = (4 - (bytes.len() % 4)) % 4;
        buf.extend_from_slice(&[0u8; 4][..pad]);
    }

    fn encode_u32(&self, v: u32, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&v.to_be_bytes());
    }

    fn encode_u64(&self, v: u64, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&v.to_be_bytes());
    }
}

/// NFS client statistics
pub struct NfsClientStats {
    /// Total RPC calls
    pub rpc_calls: AtomicU64,
    /// Total RPC retransmissions
    pub rpc_retrans: AtomicU64,
    /// Total RPC timeouts
    pub rpc_timeouts: AtomicU64,
    /// Read operations
    pub reads: AtomicU64,
    /// Write operations
    pub writes: AtomicU64,
    /// Bytes read
    pub bytes_read: AtomicU64,
    /// Bytes written
    pub bytes_written: AtomicU64,
}

impl NfsClientStats {
    pub const fn new() -> Self {
        NfsClientStats {
            rpc_calls: AtomicU64::new(0),
            rpc_retrans: AtomicU64::new(0),
            rpc_timeouts: AtomicU64::new(0),
            reads: AtomicU64::new(0),
            writes: AtomicU64::new(0),
            bytes_read: AtomicU64::new(0),
            bytes_written: AtomicU64::new(0),
        }
    }
}

// ============================================================================
// Standalone XDR encoding/decoding helpers
// ============================================================================

/// XDR encode a u32 (big-endian) into a buffer
#[inline]
pub fn xdr_encode_u32(v: u32, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&v.to_be_bytes());
}

/// XDR encode a u64 (big-endian) into a buffer
#[inline]
pub fn xdr_encode_u64(v: u64, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&v.to_be_bytes());
}

/// XDR decode a u32 from a byte slice at the given offset.
/// Returns the decoded value and the next offset.
pub fn xdr_decode_u32(buf: &[u8], offset: usize) -> Option<(u32, usize)> {
    if offset + 4 > buf.len() {
        return None;
    }
    let val = u32::from_be_bytes([buf[offset], buf[offset+1], buf[offset+2], buf[offset+3]]);
    Some((val, offset + 4))
}

/// XDR decode a u64 from a byte slice at the given offset.
/// Returns the decoded value and the next offset.
pub fn xdr_decode_u64(buf: &[u8], offset: usize) -> Option<(u64, usize)> {
    if offset + 8 > buf.len() {
        return None;
    }
    let val = u64::from_be_bytes([
        buf[offset], buf[offset+1], buf[offset+2], buf[offset+3],
        buf[offset+4], buf[offset+5], buf[offset+6], buf[offset+7],
    ]);
    Some((val, offset + 8))
}

/// XDR encode a variable-length opaque byte sequence.
pub fn xdr_encode_opaque(data: &[u8], buf: &mut Vec<u8>) {
    xdr_encode_u32(data.len() as u32, buf);
    buf.extend_from_slice(data);
    let pad = (4 - (data.len() % 4)) % 4;
    if pad > 0 {
        buf.extend_from_slice(&[0u8; 3][..pad]);
    }
}

/// XDR encode a string (same wire format as opaque).
pub fn xdr_encode_string(s: &str, buf: &mut Vec<u8>) {
    xdr_encode_opaque(s.as_bytes(), buf);
}

/// XDR decode a variable-length opaque from a byte slice.
/// Returns the slice reference into the original buffer and the next offset.
pub fn xdr_decode_opaque<'a>(buf: &'a [u8], offset: usize) -> Option<(&'a [u8], usize)> {
    let (len, data_start) = xdr_decode_u32(buf, offset)?;
    let len = len as usize;
    let padded = data_start + ((len + 3) / 4) * 4;
    if padded > buf.len() {
        return None;
    }
    Some((&buf[data_start..data_start + len], padded))
}

/// XDR encode a file handle.
pub fn xdr_encode_fh(fh: &NfsFileHandle, buf: &mut Vec<u8>) {
    xdr_encode_u32(fh.len, buf);
    buf.extend_from_slice(fh.as_bytes());
    let pad = (4 - (fh.len as usize % 4)) % 4;
    if pad > 0 {
        buf.extend_from_slice(&[0u8; 3][..pad]);
    }
}

/// XDR decode a file handle from a byte slice at offset.
/// Returns the file handle and the next offset.
pub fn xdr_decode_fh(buf: &[u8], offset: usize) -> (NfsFileHandle, usize) {
    let mut fh = NfsFileHandle::new();
    if offset + 4 > buf.len() {
        return (fh, offset);
    }
    let len = u32::from_be_bytes([buf[offset], buf[offset+1], buf[offset+2], buf[offset+3]]) as usize;
    let len = len.min(64);
    let data_start = offset + 4;
    if data_start + len <= buf.len() {
        fh.data[..len].copy_from_slice(&buf[data_start..data_start + len]);
        fh.len = len as u32;
    }
    let padded = ((len + 3) / 4) * 4;
    (fh, data_start + padded)
}

/// XDR encode an NFS time value.
pub fn xdr_encode_nfstime(t: &NfsTime, buf: &mut Vec<u8>) {
    xdr_encode_u64(t.seconds, buf);
    xdr_encode_u32(t.nseconds, buf);
}

/// XDR decode an NFS time value from a byte slice.
pub fn xdr_decode_nfstime(buf: &[u8], offset: usize) -> (NfsTime, usize) {
    if offset + 12 > buf.len() {
        return (NfsTime::new(), offset);
    }
    let seconds = u64::from_be_bytes([
        buf[offset], buf[offset+1], buf[offset+2], buf[offset+3],
        buf[offset+4], buf[offset+5], buf[offset+6], buf[offset+7],
    ]);
    let nseconds = u32::from_be_bytes([buf[offset+8], buf[offset+9], buf[offset+10], buf[offset+11]]);
    (NfsTime { seconds, nseconds }, offset + 12)
}

/// XDR decode NfsFattr from a byte slice at offset.
/// Returns the attributes and the next offset.
pub fn xdr_decode_fattr(buf: &[u8], offset: usize) -> (NfsFattr, usize) {
    let default = NfsFattr {
        ftype: NfsFileType::Reg, mode: 0, nlink: 0, uid: 0, gid: 0,
        size: 0, used: 0, rdev: NfsSpecData { specdata1: 0, specdata2: 0 },
        fsid: 0, fileid: 0, atime: NfsTime::new(), mtime: NfsTime::new(), ctime: NfsTime::new(),
    };
    if offset + 84 > buf.len() {
        return (default, offset);
    }
    let ftype_val = u32::from_be_bytes([buf[offset], buf[offset+1], buf[offset+2], buf[offset+3]]);
    let ftype = match ftype_val {
        1 => NfsFileType::Reg, 2 => NfsFileType::Dir, 3 => NfsFileType::Blk,
        4 => NfsFileType::Chr, 5 => NfsFileType::Lnk, 6 => NfsFileType::Sock,
        7 => NfsFileType::Fifo, _ => NfsFileType::Reg,
    };
    let mode = u32::from_be_bytes([buf[offset+4], buf[offset+5], buf[offset+6], buf[offset+7]]);
    let nlink = u32::from_be_bytes([buf[offset+8], buf[offset+9], buf[offset+10], buf[offset+11]]);
    let uid = u32::from_be_bytes([buf[offset+12], buf[offset+13], buf[offset+14], buf[offset+15]]);
    let gid = u32::from_be_bytes([buf[offset+16], buf[offset+17], buf[offset+18], buf[offset+19]]);
    let size = u64::from_be_bytes([
        buf[offset+20], buf[offset+21], buf[offset+22], buf[offset+23],
        buf[offset+24], buf[offset+25], buf[offset+26], buf[offset+27],
    ]);
    let used = u64::from_be_bytes([
        buf[offset+28], buf[offset+29], buf[offset+30], buf[offset+31],
        buf[offset+32], buf[offset+33], buf[offset+34], buf[offset+35],
    ]);
    let rdev1 = u32::from_be_bytes([buf[offset+36], buf[offset+37], buf[offset+38], buf[offset+39]]);
    let rdev2 = u32::from_be_bytes([buf[offset+40], buf[offset+41], buf[offset+42], buf[offset+43]]);
    let fsid = u64::from_be_bytes([
        buf[offset+44], buf[offset+45], buf[offset+46], buf[offset+47],
        buf[offset+48], buf[offset+49], buf[offset+50], buf[offset+51],
    ]);
    let fileid = u64::from_be_bytes([
        buf[offset+52], buf[offset+53], buf[offset+54], buf[offset+55],
        buf[offset+56], buf[offset+57], buf[offset+58], buf[offset+59],
    ]);
    let (atime, _) = xdr_decode_nfstime(buf, offset + 60);
    let (mtime, _) = xdr_decode_nfstime(buf, offset + 68);
    let (ctime, _) = xdr_decode_nfstime(buf, offset + 76);
    (
        NfsFattr { ftype, mode, nlink, uid, gid, size, used,
            rdev: NfsSpecData { specdata1: rdev1, specdata2: rdev2 },
            fsid, fileid, atime, mtime, ctime,
        },
        offset + 84,
    )
}

// ============================================================================
// NfsRpcClient — full RPC client with auth_unix credential support
// ============================================================================

/// RPC auth_unix credential (flavor = 1)
#[repr(C)]
pub struct RpcAuthUnix {
    /// Stamp (arbitrary, for server to detect replays)
    pub stamp: u32,
    /// Machine name
    pub machine_name: String,
    /// Effective UID
    pub uid: u32,
    /// Effective GID
    pub gid: u32,
    /// Supplementary GIDs
    pub gids: Vec<u32>,
}

impl Default for RpcAuthUnix {
    fn default() -> Self {
        RpcAuthUnix {
            stamp: 0,
            machine_name: String::new(),
            uid: 0,
            gid: 0,
            gids: Vec::new(),
        }
    }
}

impl RpcAuthUnix {
    /// XDR encode the auth_unix credential into a buffer.
    /// Format: stamp, machine_name (string), uid, gid, gids (array of u32)
    pub fn xdr_encode(&self, buf: &mut Vec<u8>) {
        // Auth flavor
        xdr_encode_u32(RpcAuthFlavor::Unix as u32, buf);
        // Credential body length placeholder — we will back-patch
        let body_pos = buf.len();
        xdr_encode_u32(0, buf);
        let body_start = buf.len();

        // stamp
        xdr_encode_u32(self.stamp, buf);
        // machine name (string)
        xdr_encode_string(&self.machine_name, buf);
        // uid
        xdr_encode_u32(self.uid, buf);
        // gid
        xdr_encode_u32(self.gid, buf);
        // gids (XDR array: length + elements)
        xdr_encode_u32(self.gids.len() as u32, buf);
        for &g in &self.gids {
            xdr_encode_u32(g, buf);
        }

        // Back-patch body length
        let body_end = buf.len();
        let body_len = (body_end - body_start) as u32;
        let len_bytes = body_len.to_be_bytes();
        buf[body_pos] = len_bytes[0];
        buf[body_pos + 1] = len_bytes[1];
        buf[body_pos + 2] = len_bytes[2];
        buf[body_pos + 3] = len_bytes[3];

        // Verifier — AUTH_NONE
        xdr_encode_u32(RpcAuthFlavor::None as u32, buf);
        xdr_encode_u32(0, buf);
    }

    /// Create a simple auth_unix with (uid, gid) and no supplementary groups.
    pub fn simple(uid: u32, gid: u32, hostname: &str) -> Self {
        RpcAuthUnix {
            stamp: 0,
            machine_name: String::from(hostname),
            uid,
            gid,
            gids: Vec::new(),
        }
    }
}

/// NfsRpcClient manages a full ONC RPC v2 connection to an NFS server,
/// supporting both AUTH_NONE and AUTH_UNIX credentials and proper
/// RPC record marking (TCP framing), XID generation, and response parsing.
pub struct NfsRpcClient {
    /// Underlying NFS client transport
    pub client: NfsClient,
    /// Authentication credential
    pub auth: RpcAuthUnix,
    /// Auth_flavor used for RPC messages
    pub auth_flavor: RpcAuthFlavor,
}

impl NfsRpcClient {
    /// Create a new RPC client connected to the given server.
    pub fn new(addr: u32, port: u16, params: &NfsMountParams) -> Self {
        NfsRpcClient {
            client: NfsClient::new(addr, port, params),
            auth: RpcAuthUnix::simple(0, 0, "nuva"),
            auth_flavor: RpcAuthFlavor::None,
        }
    }

    /// Create a new RPC client with AUTH_UNIX credentials.
    pub fn new_with_auth(addr: u32, port: u16, params: &NfsMountParams, auth: RpcAuthUnix) -> Self {
        NfsRpcClient {
            client: NfsClient::new(addr, port, params),
            auth,
            auth_flavor: RpcAuthFlavor::Unix,
        }
    }

    /// Establish transport connection.
    pub fn connect(&mut self) -> Result<(), NfsStatus> {
        self.client.connect_transport()
    }

    /// Allocate the next XID.
    pub fn next_xid(&self) -> u32 {
        self.client.xid_counter.fetch_add(1, Ordering::AcqRel)
    }

    /// Build a complete RPC call message, including the RPC header,
    /// credential, verifier, and procedure-specific arguments.
    /// Returns the assembled call buffer ready for transmission.
    pub fn build_rpc_call(&self, xid: u32, program: u32, version: u32, procedure: u32, args: &[u8]) -> Vec<u8> {
        let mut buf = Vec::new();

        // RPC header
        xdr_encode_u32(xid, &mut buf);                   // XID
        xdr_encode_u32(0, &mut buf);                      // msg_type = CALL
        xdr_encode_u32(2, &mut buf);                      // rpc_version = 2
        xdr_encode_u32(program, &mut buf);                // program
        xdr_encode_u32(version, &mut buf);                // version
        xdr_encode_u32(procedure, &mut buf);               // procedure

        // Credential
        match self.auth_flavor {
            RpcAuthFlavor::None => {
                xdr_encode_u32(RpcAuthFlavor::None as u32, &mut buf);
                xdr_encode_u32(0, &mut buf);  // zero-length body
            }
            RpcAuthFlavor::Unix => {
                self.auth.xdr_encode(&mut buf);
                // auth.xdr_encode() already includes verifier — skip below
                buf.extend_from_slice(args);
                return buf;
            }
            _ => {
                xdr_encode_u32(self.auth_flavor as u32, &mut buf);
                xdr_encode_u32(0, &mut buf);
            }
        }

        // Verifier — AUTH_NONE
        xdr_encode_u32(RpcAuthFlavor::None as u32, &mut buf);
        xdr_encode_u32(0, &mut buf);

        // Procedure-specific arguments
        buf.extend_from_slice(args);

        buf
    }

    /// Send an RPC call and receive the reply.
    pub fn send_rpc(&mut self, call_buf: &[u8]) -> Result<Vec<u8>, NfsStatus> {
        self.client.rpc_call(call_buf)
    }

    /// Parse the RPC reply header and validate the expected XID.
    /// Returns the offset to the NFS procedure result (past the RPC and NFS accept headers).
    pub fn parse_rpc_reply(&self, reply: &[u8], expected_xid: u32) -> Result<usize, NfsStatus> {
        if reply.len() < 24 {
            log_warn!("NFS: RPC reply too short: {} bytes", reply.len());
            return Err(NfsStatus::ErrIo);
        }

        // XID (4 bytes)
        let (xid, off) = xdr_decode_u32(reply, 0).ok_or(NfsStatus::ErrIo)?;
        if xid != expected_xid {
            log_warn!("NFS: XID mismatch: got {} expected {}", xid, expected_xid);
            return Err(NfsStatus::ErrIo);
        }

        // msg_type (4 bytes) — must be REPLY (1)
        let (msg_type, off) = xdr_decode_u32(reply, off).ok_or(NfsStatus::ErrIo)?;
        if msg_type != RPC_MSG_REPLY {
            log_warn!("NFS: expected REPLY (1), got {}", msg_type);
            return Err(NfsStatus::ErrIo);
        }

        // reply_stat (4 bytes) — MSG_ACCEPTED = 0
        let (reply_stat, off) = xdr_decode_u32(reply, off).ok_or(NfsStatus::ErrIo)?;
        if reply_stat != 0 {
            // MSG_DENIED
            let (reject_stat, _) = xdr_decode_u32(reply, off).ok_or(NfsStatus::ErrIo)?;
            log_warn!("NFS: RPC call denied, reject_stat={}", reject_stat);
            return Err(NfsStatus::ErrAcces);
        }

        // auth verifier (flavor + length + body)
        let (verf_flavor, off) = xdr_decode_u32(reply, off).ok_or(NfsStatus::ErrIo)?;
        let (verf_len, off) = xdr_decode_u32(reply, off).ok_or(NfsStatus::ErrIo)?;
        let verf_body_padded = (((verf_len as usize) + 3) / 4) * 4;
        let off = off + verf_body_padded;

        // accept_stat (4 bytes) — SUCCESS = 0
        let (accept_stat, off) = xdr_decode_u32(reply, off).ok_or(NfsStatus::ErrIo)?;
        if accept_stat != RPC_ACCEPT_OK {
            // Map accept_stat codes
            let err = match accept_stat {
                1 => NfsStatus::ErrPerm,    // PROG_UNAVAIL
                2 => NfsStatus::ErrInval,   // PROG_MISMATCH
                3 => NfsStatus::ErrInval,   // PROC_UNAVAIL
                4 => NfsStatus::ErrInval,   // GARBAGE_ARGS
                5 => NfsStatus::ErrIo,      // SYSTEM_ERR
                _ => NfsStatus::ErrIo,
            };
            log_warn!("NFS: RPC accept_stat error: {}", accept_stat);
            return Err(err);
        }

        // offset now points to the NFS procedure result
        Ok(off)
    }

    /// Decode the NFS status from the procedure result.
    /// Returns the offset past the status field, or an error if the status is non-OK.
    pub fn decode_nfs_status(&self, buf: &[u8], offset: usize) -> Result<usize, NfsStatus> {
        let (status, off) = xdr_decode_u32(buf, offset).ok_or(NfsStatus::ErrIo)?;
        if status == 0 {
            Ok(off)
        } else {
            Err(match status {
                1 => NfsStatus::ErrPerm,
                2 => NfsStatus::ErrNoent,
                5 => NfsStatus::ErrIo,
                6 => NfsStatus::ErrNxio,
                13 => NfsStatus::ErrAcces,
                17 => NfsStatus::ErrExist,
                18 => NfsStatus::ErrXdev,
                20 => NfsStatus::ErrNotdir,
                21 => NfsStatus::ErrIsdir,
                22 => NfsStatus::ErrInval,
                27 => NfsStatus::ErrFbig,
                28 => NfsStatus::ErrNospc,
                30 => NfsStatus::ErrRoFs,
                31 => NfsStatus::ErrMlink,
                63 => NfsStatus::ErrNametoolong,
                66 => NfsStatus::ErrNotempty,
                69 => NfsStatus::ErrDquot,
                70 => NfsStatus::ErrStale,
                10001 => NfsStatus::ErrBadHandle,
                10003 => NfsStatus::ErrBadCookie,
                10004 => NfsStatus::ErrNotSync,
                10007 => NfsStatus::ErrBadType,
                10008 => NfsStatus::ErrJukebox,
                _ => NfsStatus::ErrIo,
            })
        }
    }
}

// ============================================================================
// Standalone NFS RPC operation functions
// These use an NfsRpcClient to perform common NFS v3 operations.
// ============================================================================

/// NULL RPC probe — verifies server connectivity without performing any file operation.
/// Returns Ok(()) if the server responds correctly.
pub fn nfs_null_probe(rpc: &mut NfsRpcClient) -> Result<(), NfsStatus> {
    rpc.connect()?;

    let xid = rpc.next_xid();
    let call_buf = rpc.build_rpc_call(
        xid,
        100003,             // NFS program
        3,                  // NFS v3
        nfs3_proc::NULL,    // NULL procedure
        b"",                // no args
    );

    let reply = rpc.send_rpc(&call_buf)?;
    rpc.parse_rpc_reply(&reply, xid)?;

    log_info!("NFS: NULL probe succeeded for {:#x}:{}", rpc.client.server_addr, rpc.client.server_port);
    Ok(())
}

/// RPC program numbers
pub mod rpc_program {
    pub const MOUNT: u32 = 100005;
    pub const NFS: u32 = 100003;
    pub const NLM: u32 = 100021;
    pub const STATMON: u32 = 100024;
}

/// MOUNT protocol v3 procedure numbers
pub mod mount3_proc {
    pub const NULL: u32 = 0;
    pub const MNT: u32 = 1;
    pub const DUMP: u32 = 2;
    pub const UMNT: u32 = 3;
    pub const UMNTALL: u32 = 4;
    pub const EXPORT: u32 = 5;
}

/// Mount a remote NFS export via the MOUNT protocol v3.
/// Sends a MOUNT RPC call to get the root file handle.
/// Returns the root file handle on success.
pub fn nfs_mount(rpc: &mut NfsRpcClient, export_path: &str) -> Result<NfsFileHandle, NfsStatus> {
    rpc.connect()?;
    rpc.client.state.store(nfs_client_state::MOUNTING, Ordering::Release);

    log_info!("NFS: mounting {} on {:#x}:{}", export_path, rpc.client.server_addr, rpc.client.server_port);

    // --- MOUNT protocol: MNT procedure ---
    // Build arguments: dirpath (string)
    let mut args = Vec::new();
    xdr_encode_string(export_path, &mut args);

    let xid = rpc.next_xid();
    let call_buf = rpc.build_rpc_call(
        xid,
        rpc_program::MOUNT,
        3,                     // MOUNT v3
        mount3_proc::MNT,
        &args,
    );

    let reply = rpc.send_rpc(&call_buf)?;
    let off = rpc.parse_rpc_reply(&reply, xid)?;

    // MOUNT reply: status (u32) + file_handle (opaque) + flavors list
    let (status, off) = xdr_decode_u32(&reply, off).ok_or(NfsStatus::ErrIo)?;
    if status != 0 {
        log_warn!("NFS: MOUNT returned status {}", status);
        rpc.client.state.store(nfs_client_state::ERROR, Ordering::Release);
        return Err(NfsStatus::ErrPerm);
    }

    let (fh, _) = xdr_decode_fh(&reply, off);
    rpc.client.root_fh = fh;
    rpc.client.state.store(nfs_client_state::ACTIVE, Ordering::Release);

    log_info!("NFS: mounted {}, root_fh len={}", export_path, fh.len);
    Ok(fh)
}

/// Lookup a file/directory name within a directory on the NFS server.
/// Returns the file handle and attributes of the found entry.
pub fn nfs_lookup(rpc: &mut NfsRpcClient, dir_fh: &NfsFileHandle, name: &str) -> Result<(NfsFileHandle, NfsFattr), NfsStatus> {
    let xid = rpc.next_xid();
    log_debug!("NFS lookup: xid={} name={}", xid, name);

    // Build arguments: dir (fh) + name (string)
    let mut args = Vec::new();
    xdr_encode_fh(dir_fh, &mut args);
    xdr_encode_string(name, &mut args);

    let call_buf = rpc.build_rpc_call(xid, rpc_program::NFS, 3, nfs3_proc::LOOKUP, &args);
    let reply = rpc.send_rpc(&call_buf)?;
    let off = rpc.parse_rpc_reply(&reply, xid)?;
    let off = rpc.decode_nfs_status(&reply, off)?;

    // LOOKUP reply: object_fh + obj_attributes (post_op_attr)
    let (fh, off) = xdr_decode_fh(&reply, off);
    // Read post_op_attr: has_attributes (u32) + fattr3 (if true)
    let (has_attr, off) = xdr_decode_u32(&reply, off).ok_or(NfsStatus::ErrIo)?;
    let attr = if has_attr != 0 {
        let (attr, _) = xdr_decode_fattr(&reply, off);
        attr
    } else {
        NfsFattr {
            ftype: NfsFileType::Reg, mode: 0, nlink: 0, uid: 0, gid: 0,
            size: 0, used: 0, rdev: NfsSpecData { specdata1: 0, specdata2: 0 },
            fsid: 0, fileid: 0, atime: NfsTime::new(), mtime: NfsTime::new(), ctime: NfsTime::new(),
        }
    };

    Ok((fh, attr))
}

/// Read data from a file on the NFS server.
/// Returns the actual data read (may be less than `count`).
pub fn nfs_read(rpc: &mut NfsRpcClient, file_fh: &NfsFileHandle, offset: u64, count: u32) -> Result<Vec<u8>, NfsStatus> {
    let xid = rpc.next_xid();
    let actual_count = if count > rpc.client.rsize { rpc.client.rsize } else { count };
    log_debug!("NFS read: xid={} offset={} count={}", xid, offset, actual_count);

    // Build arguments: file (fh) + offset (u64) + count (u32)
    let mut args = Vec::new();
    xdr_encode_fh(file_fh, &mut args);
    xdr_encode_u64(offset, &mut args);
    xdr_encode_u32(actual_count, &mut args);

    let call_buf = rpc.build_rpc_call(xid, rpc_program::NFS, 3, nfs3_proc::READ, &args);
    let reply = rpc.send_rpc(&call_buf)?;
    let off = rpc.parse_rpc_reply(&reply, xid)?;
    let off = rpc.decode_nfs_status(&reply, off)?;

    // READ reply: attributes (post_op_attr) + count (u32) + eof (bool) + data (opaque)
    let (has_attr, mut off) = xdr_decode_u32(&reply, off).ok_or(NfsStatus::ErrIo)?;
    if has_attr != 0 {
        let (_, attr_end) = xdr_decode_fattr(&reply, off);
        off = attr_end;
    }
    let (_data_count, off) = xdr_decode_u32(&reply, off).ok_or(NfsStatus::ErrIo)?;
    let (_eof, off) = xdr_decode_u32(&reply, off).ok_or(NfsStatus::ErrIo)?;
    let (data_slice, _) = xdr_decode_opaque(&reply, off).unwrap_or((&[], off));

    Ok(data_slice.to_vec())
}

/// Write data to a file on the NFS server.
/// Returns the number of bytes actually written and the write stability used.
pub fn nfs_write(rpc: &mut NfsRpcClient, file_fh: &NfsFileHandle, offset: u64, data: &[u8], stable: NfsStableHow) -> Result<(u32, NfsStableHow), NfsStatus> {
    let xid = rpc.next_xid();
    let write_len = if data.len() as u32 > rpc.client.wsize {
        rpc.client.wsize as usize
    } else {
        data.len()
    };
    log_debug!("NFS write: xid={} offset={} len={}", xid, offset, write_len);

    // Build arguments: file (fh) + offset (u64) + count (u32) + stable (u32) + data (opaque)
    let mut args = Vec::new();
    xdr_encode_fh(file_fh, &mut args);
    xdr_encode_u64(offset, &mut args);
    xdr_encode_u32(write_len as u32, &mut args);
    xdr_encode_u32(stable as u32, &mut args);
    // data as XDR opaque
    xdr_encode_opaque(&data[..write_len], &mut args);

    let call_buf = rpc.build_rpc_call(xid, rpc_program::NFS, 3, nfs3_proc::WRITE, &args);
    let reply = rpc.send_rpc(&call_buf)?;
    let off = rpc.parse_rpc_reply(&reply, xid)?;
    let off = rpc.decode_nfs_status(&reply, off)?;

    // WRITE reply: attributes (wcc_data) + count (u32) + committed (u32) + verf (u64)
    // wcc_data: before (pre_op_attr) + after (post_op_attr)
    // pre_op_attr: has_attributes (u32) + size/u64 + mtime/u32 + ctime/u32 (if present)
    let (has_pre, mut off) = xdr_decode_u32(&reply, off).ok_or(NfsStatus::ErrIo)?;
    if has_pre != 0 {
        // size (u64)
        off += 8;
        // mtime (nfstime = u64 + u32 = 12 bytes)
        off += 12;
        // ctime (nfstime = 12 bytes)
        off += 12;
    }
    // post_op_attr
    let (has_post, off_val) = xdr_decode_u32(&reply, off).ok_or(NfsStatus::ErrIo)?;
    let off2 = if has_post != 0 {
        let (_, attr_end) = xdr_decode_fattr(&reply, off_val);
        attr_end
    } else {
        off_val
    };

    let (count, off2) = xdr_decode_u32(&reply, off2).ok_or(NfsStatus::ErrIo)?;
    let (committed, _) = xdr_decode_u32(&reply, off2).ok_or(NfsStatus::ErrIo)?;
    // verf at off2 + 4, skip for now

    let how = match committed {
        0 => NfsStableHow::Unstable,
        1 => NfsStableHow::DataSync,
        _ => NfsStableHow::FileSync,
    };

    Ok((count, how))
}

/// Get file attributes from the NFS server.
pub fn nfs_getattr(rpc: &mut NfsRpcClient, file_fh: &NfsFileHandle) -> Result<NfsFattr, NfsStatus> {
    let xid = rpc.next_xid();
    log_debug!("NFS getattr: xid={}", xid);

    // Build arguments: object (fh)
    let mut args = Vec::new();
    xdr_encode_fh(file_fh, &mut args);

    let call_buf = rpc.build_rpc_call(xid, rpc_program::NFS, 3, nfs3_proc::GETATTR, &args);
    let reply = rpc.send_rpc(&call_buf)?;
    let off = rpc.parse_rpc_reply(&reply, xid)?;
    let off = rpc.decode_nfs_status(&reply, off)?;

    // GETATTR reply: attributes (fattr3)
    let (attr, _) = xdr_decode_fattr(&reply, off);
    Ok(attr)
}

/// Convert an NfsStatus to a POSIX errno value.
pub fn nfs_status_to_errno(status: NfsStatus) -> i32 {
    match status {
        NfsStatus::Ok => 0,
        NfsStatus::ErrPerm => -1,       // EPERM
        NfsStatus::ErrNoent => -2,      // ENOENT
        NfsStatus::ErrIo => -5,         // EIO
        NfsStatus::ErrNxio => -6,       // ENXIO
        NfsStatus::ErrAcces => -13,     // EACCES
        NfsStatus::ErrExist => -17,     // EEXIST
        NfsStatus::ErrXdev => -18,      // EXDEV
        NfsStatus::ErrNotdir => -20,    // ENOTDIR
        NfsStatus::ErrIsdir => -21,     // EISDIR
        NfsStatus::ErrInval => -22,     // EINVAL
        NfsStatus::ErrFbig => -27,      // EFBIG
        NfsStatus::ErrNospc => -28,     // ENOSPC
        NfsStatus::ErrRoFs => -30,      // EROFS
        NfsStatus::ErrMlink => -31,     // EMLINK
        NfsStatus::ErrNametoolong => -36, // ENAMETOOLONG
        NfsStatus::ErrNotempty => -39,  // ENOTEMPTY
        NfsStatus::ErrDquot => -122,    // EDQUOT
        NfsStatus::ErrStale => -116,    // ESTALE
        NfsStatus::ErrBadHandle => -100, // EBADH
        _ => -5, // EIO
    }
}

/// Mount entry point — combines MOUNT protocol with NFS probe.
/// Returns the root file handle for the mounted export.
pub fn nfs_mount_and_probe(addr: u32, port: u16, params: &NfsMountParams) -> Result<(NfsRpcClient, NfsFileHandle), NfsStatus> {
    let mut rpc = NfsRpcClient::new(addr, port, params);

    // First, probe server with a NULL RPC
    nfs_null_probe(&mut rpc)?;

    // Then mount the export
    let root_fh = nfs_mount(&mut rpc, &params.export_path)?;

    Ok((rpc, root_fh))
}
