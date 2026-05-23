/*
 * Nuva OS - Kernel - Kernel
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


pub mod net;
pub mod vfs;

/// BSD compatibility layer version
pub const BSD_COMPAT_VERSION: &str = "1.0.0";

/// BSD error codes
pub mod errno {
    pub const EPERM: i32 = 1;
    pub const ENOENT: i32 = 2;
    pub const ESRCH: i32 = 3;
    pub const EINTR: i32 = 4;
    pub const EIO: i32 = 5;
    pub const ENXIO: i32 = 6;
    pub const E2BIG: i32 = 7;
    pub const ENOEXEC: i32 = 8;
    pub const EBADF: i32 = 9;
    pub const ECHILD: i32 = 10;
    pub const EDEADLK: i32 = 11;
    pub const ENOMEM: i32 = 12;
    pub const EACCES: i32 = 13;
    pub const EFAULT: i32 = 14;
    pub const ENOTBLK: i32 = 15;
    pub const EBUSY: i32 = 16;
    pub const EEXIST: i32 = 17;
    pub const EXDEV: i32 = 18;
    pub const ENODEV: i32 = 19;
    pub const ENOTDIR: i32 = 20;
    pub const EISDIR: i32 = 21;
    pub const EINVAL: i32 = 22;
    pub const ENFILE: i32 = 23;
    pub const EMFILE: i32 = 24;
    pub const ENOTTY: i32 = 25;
    pub const ETXTBSY: i32 = 26;
    pub const EFBIG: i32 = 27;
    pub const ENOSPC: i32 = 28;
    pub const ESPIPE: i32 = 29;
    pub const EROFS: i32 = 30;
    pub const EMLINK: i32 = 31;
    pub const EPIPE: i32 = 32;
    pub const EDOM: i32 = 33;
    pub const ERANGE: i32 = 34;
    pub const EAGAIN: i32 = 35;
    pub const EWOULDBLOCK: i32 = EAGAIN;
    pub const EINPROGRESS: i32 = 36;
    pub const EALREADY: i32 = 37;
    pub const ENOTSOCK: i32 = 38;
    pub const EDESTADDRREQ: i32 = 39;
    pub const EMSGSIZE: i32 = 40;
    pub const EPROTOTYPE: i32 = 41;
    pub const ENOPROTOOPT: i32 = 42;
    pub const EPROTONOSUPPORT: i32 = 43;
    pub const ESOCKTNOSUPPORT: i32 = 44;
    pub const EOPNOTSUPP: i32 = 45;
    pub const EPFNOSUPPORT: i32 = 46;
    pub const EAFNOSUPPORT: i32 = 47;
    pub const EADDRINUSE: i32 = 48;
    pub const EADDRNOTAVAIL: i32 = 49;
    pub const ENETDOWN: i32 = 50;
    pub const ENETUNREACH: i32 = 51;
    pub const ENETRESET: i32 = 52;
    pub const ECONNABORTED: i32 = 53;
    pub const ECONNRESET: i32 = 54;
    pub const ENOBUFS: i32 = 55;
    pub const EISCONN: i32 = 56;
    pub const ENOTCONN: i32 = 57;
    pub const ESHUTDOWN: i32 = 58;
    pub const ETOOMANYREFS: i32 = 59;
    pub const ETIMEDOUT: i32 = 60;
    pub const ECONNREFUSED: i32 = 61;
    pub const ELOOP: i32 = 62;
    pub const ENAMETOOLONG: i32 = 63;
    pub const EHOSTDOWN: i32 = 64;
    pub const EHOSTUNREACH: i32 = 65;
    pub const ENOSYS: i32 = 78;
}

/// BSD FileDescriptorFlag
pub mod fcntl {
    pub const O_RDONLY: i32 = 0;
    pub const O_WRONLY: i32 = 1;
    pub const O_RDWR: i32 = 2;
    pub const O_ACCMODE: i32 = 3;
    pub const O_CREAT: i32 = 0x0200;
    pub const O_EXCL: i32 = 0x0800;
    pub const O_NOCTTY: i32 = 0x8000;
    pub const O_TRUNC: i32 = 0x0400;
    pub const O_APPEND: i32 = 0x0008;
    pub const O_NONBLOCK: i32 = 0x0004;
    pub const O_SYNC: i32 = 0x0080;
    pub const O_FSYNC: i32 = O_SYNC;
    pub const O_ASYNC: i32 = 0x0040;
    pub const O_SHLOCK: i32 = 0x0010;
    pub const O_EXLOCK: i32 = 0x0020;
    pub const O_NOFOLLOW: i32 = 0x0100;
    pub const O_SYMLINK: i32 = 0x200000;
    pub const O_EVTONLY: i32 = 0x8000;
    pub const O_CLOEXEC: i32 = 0x1000000;
}

/// BSD file modes
pub mod stat {
    pub const S_IFMT: u32 = 0o170000;
    pub const S_IFIFO: u32 = 0o010000;
    pub const S_IFCHR: u32 = 0o020000;
    pub const S_IFDIR: u32 = 0o040000;
    pub const S_IFBLK: u32 = 0o060000;
    pub const S_IFREG: u32 = 0o100000;
    pub const S_IFLNK: u32 = 0o120000;
    pub const S_IFSOCK: u32 = 0o140000;
    pub const S_ISUID: u32 = 0o4000;
    pub const S_ISGID: u32 = 0o2000;
    pub const S_ISVTX: u32 = 0o1000;
    pub const S_IRWXU: u32 = 0o0700;
    pub const S_IRUSR: u32 = 0o0400;
    pub const S_IWUSR: u32 = 0o0200;
    pub const S_IXUSR: u32 = 0o0100;
    pub const S_IRWXG: u32 = 0o0070;
    pub const S_IRGRP: u32 = 0o0040;
    pub const S_IWGRP: u32 = 0o0020;
    pub const S_IXGRP: u32 = 0o0010;
    pub const S_IRWXO: u32 = 0o0007;
    pub const S_IROTH: u32 = 0o0004;
    pub const S_IWOTH: u32 = 0o0002;
    pub const S_IXOTH: u32 = 0o0001;
    
    pub const S_ISDIR: u32 = 0o040000;
    pub const S_ISCHR: u32 = 0o020000;
    pub const S_ISBLK: u32 = 0o060000;
    pub const S_ISREG: u32 = 0o100000;
    pub const S_ISFIFO: u32 = 0o010000;
    pub const S_ISLNK: u32 = 0o120000;
    pub const S_ISSOCK: u32 = 0o140000;
}

/// BSD Socket Constant
pub mod socket {
    // Address families
    pub const AF_UNSPEC: i32 = 0;
    pub const AF_UNIX: i32 = 1;
    pub const AF_INET: i32 = 2;
    pub const AF_INET6: i32 = 30;
    pub const AF_ROUTE: i32 = 17;
    pub const AF_LINK: i32 = 18;
    
    // Socket types
    pub const SOCK_STREAM: i32 = 1;
    pub const SOCK_DGRAM: i32 = 2;
    pub const SOCK_RAW: i32 = 3;
    pub const SOCK_SEQPACKET: i32 = 5;
    
    // Protocols
    pub const IPPROTO_IP: i32 = 0;
    pub const IPPROTO_ICMP: i32 = 1;
    pub const IPPROTO_TCP: i32 = 6;
    pub const IPPROTO_UDP: i32 = 17;
    pub const IPPROTO_IPV6: i32 = 41;
    
    // Socket options
    pub const SOL_SOCKET: i32 = 0xffff;
    pub const SO_DEBUG: i32 = 0x0001;
    pub const SO_ACCEPTCONN: i32 = 0x0002;
    pub const SO_REUSEADDR: i32 = 0x0004;
    pub const SO_KEEPALIVE: i32 = 0x0008;
    pub const SO_DONTROUTE: i32 = 0x0010;
    pub const SO_BROADCAST: i32 = 0x0020;
    pub const SO_USELOOPBACK: i32 = 0x0040;
    pub const SO_LINGER: i32 = 0x0080;
    pub const SO_OOBINLINE: i32 = 0x0100;
    pub const SO_REUSEPORT: i32 = 0x0200;
    pub const SO_SNDBUF: i32 = 0x1001;
    pub const SO_RCVBUF: i32 = 0x1002;
    pub const SO_SNDLOWAT: i32 = 0x1003;
    pub const SO_RCVLOWAT: i32 = 0x1004;
    pub const SO_SNDTIMEO: i32 = 0x1005;
    pub const SO_RCVTIMEO: i32 = 0x1006;
    pub const SO_ERROR: i32 = 0x1007;
    pub const SO_TYPE: i32 = 0x1008;
    
    // Message flags
    pub const MSG_OOB: i32 = 0x1;
    pub const MSG_PEEK: i32 = 0x2;
    pub const MSG_DONTROUTE: i32 = 0x4;
    pub const MSG_EOR: i32 = 0x8;
    pub const MSG_TRUNC: i32 = 0x10;
    pub const MSG_CTRUNC: i32 = 0x20;
    pub const MSG_WAITALL: i32 = 0x40;
    pub const MSG_DONTWAIT: i32 = 0x80;
    pub const MSG_EOF: i32 = 0x100;
    
    // Shutdown
    pub const SHUT_RD: i32 = 0;
    pub const SHUT_WR: i32 = 1;
    pub const SHUT_RDWR: i32 = 2;
}

/// BSD compatibility layer initialization
pub fn init_bsd_compat() {
    // Initialize network compatibility layer
    net::init_bsd_net();
    
    // Initialize VFS compatibility layer
    vfs::init_bsd_vfs();
    
    log_info!("BSD compatibility layer initialized");
    log_info!("  Version: {}", BSD_COMPAT_VERSION);
}