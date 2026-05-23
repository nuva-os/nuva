/*
 * Nuva OS - NuvaFS POSIX Compliance Layer
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

//! NuvaFS POSIX Compliance Layer
/*!*/
//! Implements POSIX file system operations and error codes.

use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};

/// POSIX error codes (errno)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Errno {
    Success = 0,
    EPERM = 1,
    ENOENT = 2,
    ESRCH = 3,
    EINTR = 4,
    EIO = 5,
    ENXIO = 6,
    E2BIG = 7,
    ENOEXEC = 8,
    EBADF = 9,
    ECHILD = 10,
    EAGAIN = 11,
    ENOMEM = 12,
    EACCES = 13,
    EFAULT = 14,
    ENOTBLK = 15,
    EBUSY = 16,
    EEXIST = 17,
    EXDEV = 18,
    ENODEV = 19,
    ENOTDIR = 20,
    EISDIR = 21,
    EINVAL = 22,
    ENFILE = 23,
    EMFILE = 24,
    ENOTTY = 25,
    ETXTBSY = 26,
    EFBIG = 27,
    ENOSPC = 28,
    ESPIPE = 29,
    EROFS = 30,
    EMLINK = 31,
    EPIPE = 32,
    EDOM = 33,
    ERANGE = 34,
    EDEADLK = 35,
    ENAMETOOLONG = 36,
    ENOLCK = 37,
    ENOSYS = 38,
    ENOTEMPTY = 39,
    ELOOP = 40,
    EWOULDBLOCK = 41,
    ENOMSG = 42,
    EIDRM = 43,
    ECHRNG = 44,
    EL2NSYNC = 45,
    EL3HLT = 46,
    EL3RST = 47,
    ELNRNG = 48,
    EUNATCH = 49,
    ENOCSI = 50,
    EL2HLT = 51,
    EBADE = 52,
    EBADR = 53,
    EXFULL = 54,
    ENOANO = 55,
    EBADRQC = 56,
    EBADSLT = 57,
    EDEADLOCK = 58,
    EBFONT = 59,
    ENOSTR = 60,
    ENODATA = 61,
    ETIME = 62,
    ENOSR = 63,
    ENONET = 64,
    ENOPKG = 65,
    EREMOTE = 66,
    ENOLINK = 67,
    EADV = 68,
    ESRMNT = 69,
    ECOMM = 70,
    EPROTO = 71,
    EMULTIHOP = 72,
    EDOTDOT = 73,
    EBADMSG = 74,
    EOVERFLOW = 75,
    ENOTUNIQ = 76,
    EBADFD = 77,
    EREMCHG = 78,
    ELIBACC = 79,
    ELIBBAD = 80,
    ELIBSCN = 81,
    ELIBMAX = 82,
    ELIBEXEC = 83,
    EILSEQ = 84,
    ERESTART = 85,
    ESTRPIPE = 86,
    EUSERS = 87,
    ENOTSOCK = 88,
    EDESTADDRREQ = 89,
    EMSGSIZE = 90,
    EPROTOTYPE = 91,
    ENOPROTOOPT = 92,
    EPROTONOSUPPORT = 93,
    ESOCKTNOSUPPORT = 94,
    EOPNOTSUPP = 95,
    EPFNOSUPPORT = 96,
    EAFNOSUPPORT = 97,
    EADDRINUSE = 98,
    EADDRNOTAVAIL = 99,
    ENETDOWN = 100,
    ENETUNREACH = 101,
    ENETRESET = 102,
    ECONNABORTED = 103,
    ECONNRESET = 104,
    ENOBUFS = 105,
    EISCONN = 106,
    ENOTCONN = 107,
    ESHUTDOWN = 108,
    ETOOMANYREFS = 109,
    ETIMEDOUT = 110,
    ECONNREFUSED = 111,
    EHOSTDOWN = 112,
    EHOSTUNREACH = 113,
    EALREADY = 114,
    EINPROGRESS = 115,
    ESTALE = 116,
    EUCLEAN = 117,
    ENOTNAM = 118,
    ENAVAIL = 119,
    EISNAM = 120,
    EREMOTEIO = 121,
    EDQUOT = 122,
    ENOMEDIUM = 123,
    EMEDIUMTYPE = 124,
    ECANCELED = 125,
    ENOKEY = 126,
    EKEYEXPIRED = 127,
    EKEYREVOKED = 128,
    EKEYREJECTED = 129,
    EOWNERDEAD = 130,
    ENOTRECOVERABLE = 131,
    ERFKILL = 132,
    EHWPOISON = 133,
}

/// POSIX file types and permissions (mode_t)
pub mod mode {
    /// File types
    pub const S_IFMT: u32 = 0o170000;
    pub const S_IFSOCK: u32 = 0o140000;
    pub const S_IFLNK: u32 = 0o120000;
    pub const S_IFREG: u32 = 0o100000;
    pub const S_IFBLK: u32 = 0o060000;
    pub const S_IFDIR: u32 = 0o040000;
    pub const S_IFCHR: u32 = 0o020000;
    pub const S_IFIFO: u32 = 0o010000;

    /// Permissions
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

    /// Helper functions
    pub fn S_ISREG(m: u32) -> bool { (m & S_IFMT) == S_IFREG }
    pub fn S_ISDIR(m: u32) -> bool { (m & S_IFMT) == S_IFDIR }
    pub fn S_ISLNK(m: u32) -> bool { (m & S_IFMT) == S_IFLNK }
    pub fn S_ISBLK(m: u32) -> bool { (m & S_IFMT) == S_IFBLK }
    pub fn S_ISCHR(m: u32) -> bool { (m & S_IFMT) == S_IFCHR }
    pub fn S_ISFIFO(m: u32) -> bool { (m & S_IFMT) == S_IFIFO }
    pub fn S_ISSOCK(m: u32) -> bool { (m & S_IFMT) == S_IFSOCK }
}

/// POSIX open flags
pub mod open_flags {
    pub const O_RDONLY: i32 = 0o0;
    pub const O_WRONLY: i32 = 0o1;
    pub const O_RDWR: i32 = 0o2;
    pub const O_CREAT: i32 = 0o100;
    pub const O_EXCL: i32 = 0o200;
    pub const O_NOCTTY: i32 = 0o400;
    pub const O_TRUNC: i32 = 0o1000;
    pub const O_APPEND: i32 = 0o2000;
    pub const O_NONBLOCK: i32 = 0o4000;
    pub const O_SYNC: i32 = 0o10000;
    pub const O_ASYNC: i32 = 0o20000;
    pub const O_DIRECT: i32 = 0o40000;
    pub const O_LARGEFILE: i32 = 0o100000;
    pub const O_DIRECTORY: i32 = 0o200000;
    pub const O_NOFOLLOW: i32 = 0o400000;
    pub const O_CLOEXEC: i32 = 0o2000000;
    pub const O_PATH: i32 = 0o10000000;
}

/// POSIX seek whence
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Whence {
    SEEK_SET = 0,
    SEEK_CUR = 1,
    SEEK_END = 2,
    SEEK_DATA = 3,
    SEEK_HOLE = 4,
}

/// POSIX stat structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Stat {
    pub device_id: u64,
    pub inode_number: u64,
    pub mode: u32,
    pub link_count: u32,
    pub user_id: u32,
    pub group_id: u32,
    pub raw_device_id: u64,
    pub size: i64,
    pub block_size: i64,
    pub block_count: i64,
    pub access_time: i64,
    pub access_time_nsec: i64,
    pub modification_time: i64,
    pub modification_time_nsec: i64,
    pub change_time: i64,
    pub change_time_nsec: i64,
}

impl Stat {
    pub fn new() -> Self {
        Self {
            device_id: 0,
            inode_number: 0,
            mode: 0,
            link_count: 0,
            user_id: 0,
            group_id: 0,
            raw_device_id: 0,
            size: 0,
            block_size: 4096,
            block_count: 0,
            access_time: 0,
            access_time_nsec: 0,
            modification_time: 0,
            modification_time_nsec: 0,
            change_time: 0,
            change_time_nsec: 0,
        }
    }
}

/// POSIX file lock structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Flock {
    pub l_type: i16,
    pub l_whence: i16,
    pub l_start: i64,
    pub l_len: i64,
    pub l_pid: i32,
}

/// Lock types
pub const F_RDLCK: i16 = 0;
pub const F_WRLCK: i16 = 1;
pub const F_UNLCK: i16 = 2;

/// POSIX dirent structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Dirent {
    pub d_ino: u64,
    pub d_off: i64,
    pub d_reclen: u16,
    pub d_type: u8,
    pub d_name: [u8; 256],
}

/// Dirent types
pub const DT_UNKNOWN: u8 = 0;
pub const DT_FIFO: u8 = 1;
pub const DT_CHR: u8 = 2;
pub const DT_DIR: u8 = 4;
pub const DT_BLK: u8 = 6;
pub const DT_REG: u8 = 8;
pub const DT_LNK: u8 = 10;
pub const DT_SOCK: u8 = 12;
pub const DT_WHT: u8 = 14;

/// POSIX permission check
pub struct PermissionCheck;

impl PermissionCheck {
    /// Check read permission
    pub fn can_read(mode: u32, uid: u32, gid: u32, file_uid: u32, file_gid: u32, file_mode: u32) -> bool {
        if uid == 0 {
            return true; // Root can read anything
        }

        if uid == file_uid {
            return (file_mode & mode::S_IRUSR) != 0;
        }

        if gid == file_gid {
            return (file_mode & mode::S_IRGRP) != 0;
        }

        (file_mode & mode::S_IROTH) != 0
    }

    /// Check write permission
    pub fn can_write(mode: u32, uid: u32, gid: u32, file_uid: u32, file_gid: u32, file_mode: u32) -> bool {
        if uid == 0 {
            return true; // Root can write anything
        }

        if uid == file_uid {
            return (file_mode & mode::S_IWUSR) != 0;
        }

        if gid == file_gid {
            return (file_mode & mode::S_IWGRP) != 0;
        }

        (file_mode & mode::S_IWOTH) != 0
    }

    /// Check execute permission
    pub fn can_execute(mode: u32, uid: u32, gid: u32, file_uid: u32, file_gid: u32, file_mode: u32) -> bool {
        if uid == 0 {
            // Root needs at least one execute bit set
            return (file_mode & (mode::S_IXUSR | mode::S_IXGRP | mode::S_IXOTH)) != 0;
        }

        if uid == file_uid {
            return (file_mode & mode::S_IXUSR) != 0;
        }

        if gid == file_gid {
            return (file_mode & mode::S_IXGRP) != 0;
        }

        (file_mode & mode::S_IXOTH) != 0
    }

    /// Check if sticky bit prevents deletion
    pub fn can_delete(
        dir_mode: u32,
        dir_uid: u32,
        dir_gid: u32,
        file_uid: u32,
        current_uid: u32,
        current_gid: u32,
    ) -> bool {
        // If sticky bit is set, only owner of file or directory can delete
        if (dir_mode & mode::S_ISVTX) != 0 {
            if current_uid == 0 {
                return true;
            }
            if current_uid == dir_uid || current_uid == file_uid {
                return true;
            }
            return false;
        }

        // Otherwise, need write permission on directory
        Self::can_write(0, current_uid, current_gid, dir_uid, dir_gid, dir_mode)
    }
}

/// POSIX path operations
pub struct PathOps;

impl PathOps {
    /// Validate path name
    pub fn validate(path: &[u8]) -> Result<(), Errno> {
        if path.is_empty() {
            return Err(Errno::ENOENT);
        }

        if path.len() > 4096 {
            return Err(Errno::ENAMETOOLONG);
        }

        // Check for null bytes
        for &b in path {
            if b == 0 {
                return Err(Errno::EINVAL);
            }
        }

        Ok(())
    }

    /// Split path into directory and filename
    pub fn split(path: &[u8]) -> (&[u8], &[u8]) {
        if path.is_empty() {
            return (&[], &[]);
        }

        // Find last separator
        let mut last_sep = 0;
        for i in 0..path.len() {
            if path[i] == b'/' {
                last_sep = i;
            }
        }

        if last_sep == 0 {
            if path[0] == b'/' {
                (&path[..1], &path[1..])
            } else {
                (&[], path)
            }
        } else {
            (&path[..last_sep], &path[last_sep + 1..])
        }
    }

    /// Get parent directory
    pub fn parent(path: &[u8]) -> &[u8] {
        let (parent, _) = Self::split(path);
        parent
    }

    /// Get filename
    pub fn filename(path: &[u8]) -> &[u8] {
        let (_, name) = Self::split(path);
        name
    }

    /// Check if path is absolute
    pub fn is_absolute(path: &[u8]) -> bool {
        !path.is_empty() && path[0] == b'/'
    }

    /// Normalize path (remove . and ..)
    pub fn normalize(path: &[u8], output: &mut [u8]) -> usize {
        let mut out_pos = 0;
        let mut components: [usize; 64] = [0; 64];
        let mut comp_count = 0;

        let mut i = 0;
        while i < path.len() {
            // Skip separators
            while i < path.len() && path[i] == b'/' {
                i += 1;
            }

            if i >= path.len() {
                break;
            }

            // Find component end
            let start = i;
            while i < path.len() && path[i] != b'/' {
                i += 1;
            }
            let component = &path[start..i];

            if component == b"." {
                // Skip
            } else if component == b".." {
                if comp_count > 0 {
                    comp_count -= 1;
                }
            } else if comp_count < 64 {
                components[comp_count] = start;
                comp_count += 1;
            }
        }

        // Build output
        for j in 0..comp_count {
            if out_pos >= output.len() {
                break;
            }
            output[out_pos] = b'/';
            out_pos += 1;

            let start = components[j];
            let mut k = start;
            while k < path.len() && path[k] != b'/' && out_pos < output.len() {
                output[out_pos] = path[k];
                out_pos += 1;
                k += 1;
            }
        }

        if out_pos == 0 {
            output[0] = b'/';
            out_pos = 1;
        }

        out_pos
    }
}

/// Current errno (thread-local)
pub static mut CURRENT_ERRNO: i32 = 0;

/// Set errno
pub fn set_errno(errno: Errno) {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        CURRENT_ERRNO = errno as i32;
    }
}

/// Get errno
pub fn get_errno() -> Errno {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        match CURRENT_ERRNO {
            0 => Errno::Success,
            1 => Errno::EPERM,
            2 => Errno::ENOENT,
            5 => Errno::EIO,
            9 => Errno::EBADF,
            12 => Errno::ENOMEM,
            13 => Errno::EACCES,
            22 => Errno::EINVAL,
            28 => Errno::ENOSPC,
            _ => Errno::EIO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_errno_values() {
        assert_eq!(Errno::ENOENT as i32, 2);
        assert_eq!(Errno::EIO as i32, 5);
        assert_eq!(Errno::ENOMEM as i32, 12);
        assert_eq!(Errno::EINVAL as i32, 22);
    }

    #[test]
    fn test_mode_checks() {
        assert!(mode::S_ISREG(mode::S_IFREG));
        assert!(mode::S_ISDIR(mode::S_IFDIR));
        assert!(mode::S_ISLNK(mode::S_IFLNK));
        assert!(!mode::S_ISREG(mode::S_IFDIR));
    }

    #[test]
    fn test_mode_permissions() {
        let mode = mode::S_IFREG | mode::S_IRUSR | mode::S_IWGRP | mode::S_IXOTH;

        assert!((mode & mode::S_IRUSR) != 0);
        assert!((mode & mode::S_IWGRP) != 0);
        assert!((mode & mode::S_IXOTH) != 0);
        assert!((mode & mode::S_IRGRP) == 0);
    }

    #[test]
    fn test_stat_new() {
        let stat = Stat::new();
        assert_eq!(stat.device_id, 0);
        assert_eq!(stat.inode_number, 0);
        assert_eq!(stat.block_size, 4096);
    }

    #[test]
    fn test_permission_check_root() {
        // Root can read/write anything
        assert!(PermissionCheck::can_read(0, 0, 0, 100, 100, 0));
        assert!(PermissionCheck::can_write(0, 0, 0, 100, 100, 0));
    }

    #[test]
    fn test_permission_check_owner() {
        let file_mode = mode::S_IRUSR | mode::S_IWUSR;

        // Owner can read/write
        assert!(PermissionCheck::can_read(0, 100, 100, 100, 100, file_mode));
        assert!(PermissionCheck::can_write(0, 100, 100, 100, 100, file_mode));

        // Others cannot
        assert!(!PermissionCheck::can_read(0, 200, 200, 100, 100, file_mode));
        assert!(!PermissionCheck::can_write(0, 200, 200, 100, 100, file_mode));
    }

    #[test]
    fn test_path_ops_validate() {
        assert!(PathOps::validate(b"/test/path").is_ok());
        assert!(PathOps::validate(b"").is_err());
        assert!(PathOps::validate(b"/test\0path").is_err());
    }

    #[test]
    fn test_path_ops_split() {
        let (dir, name) = PathOps::split(b"/home/user/file.txt");
        assert_eq!(dir, b"/home/user");
        assert_eq!(name, b"file.txt");

        let (dir, name) = PathOps::split(b"file.txt");
        assert_eq!(dir, b"");
        assert_eq!(name, b"file.txt");
    }

    #[test]
    fn test_path_ops_is_absolute() {
        assert!(PathOps::is_absolute(b"/test"));
        assert!(!PathOps::is_absolute(b"test"));
        assert!(!PathOps::is_absolute(b""));
    }

    #[test]
    fn test_path_ops_normalize() {
        let mut output = [0u8; 256];

        let len = PathOps::normalize(b"/a/b/../c", &mut output);
        assert_eq!(&output[..len], b"/a/c");

        let len = PathOps::normalize(b"/a/./b", &mut output);
        assert_eq!(&output[..len], b"/a/b");
    }
}
