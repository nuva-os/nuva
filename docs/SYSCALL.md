# Nuva OS System Call Module

## Overview

The system call module provides POSIX-compatible interfaces for user-space applications to interact with the kernel. It includes a complete system call number table, error code definitions, signal handling, and io_uring related system calls.

---

## Table of Contents

1. [System Call Architecture](#1-system-call-architecture)
2. [System Call Number Table](#2-system-call-number-table)
3. [File Operations](#3-file-operations)
4. [Process Operations](#4-process-operations)
5. [Memory Operations](#5-memory-operations)
6. [IPC Operations](#6-ipc-operations)
7. [Network Operations](#7-network-operations)
8. [Signal Handling System Calls](#8-signal-handling-system-calls)
9. [io_uring System Calls](#9-io_uring-system-calls)
10. [Error Codes](#10-error-codes)
11. [File Structure](#11-file-structure)

---

## 1. System Call Architecture

### 1.1 System Call Interface

System calls are the primary interface between user-space applications and the kernel. They provide controlled access to kernel services.

### 1.2 System Call Convention

```rust
pub type SyscallHandler = fn(u64, u64, u64, u64, u64) -> Result<u64>;
```

System calls accept up to 6 parameters (including the system call number) and return a result.

### 1.3 System Call Table

```rust
pub const SYSCALL_TABLE: &[SyscallHandler] = &[
    sys_read,       // 0
    sys_write,      // 1
    sys_open,       // 2
    sys_close,      // 3
    sys_stat,       // 4
    sys_fstat,      // 5
    sys_poll,       // 6
    sys_lseek,      // 7
    sys_mmap,       // 8
    sys_mprotect,   // 9
    sys_munmap,     // 10
    sys_brk,        // 11
    sys_ioctl,      // 12
    sys_fcntl,      // 13
    sys_fsync,      // 14
    sys_pipe,       // 15
    // ... more system calls
];
```

---

## 2. System Call Number Table

### 2.1 File I/O System Calls

| Number | System Call | Description |
|--------|-------------|-------------|
| 0 | `read` | Read from file descriptor |
| 1 | `write` | Write to file descriptor |
| 2 | `open` | Open file |
| 3 | `close` | Close file descriptor |
| 4 | `stat` | Get file status |
| 5 | `fstat` | Get file status by fd |
| 6 | `lstat` | Get symlink file status |
| 7 | `poll` | Wait for events on file descriptors |
| 8 | `lseek` | Set file offset |
| 9 | `mmap` | Map memory |
| 10 | `mprotect` | Set memory protection |
| 11 | `munmap` | Unmap memory |
| 12 | `brk` | Change data segment size |
| 13 | `rt_sigaction` | Real-time signal action setup |
| 14 | `rt_sigprocmask` | Real-time signal mask setup |
| 15 | `rt_sigreturn` | Real-time signal return |
| 16 | `ioctl` | Device control |
| 17 | `pread64` | Read at specified offset |
| 18 | `pwrite64` | Write at specified offset |
| 19 | `readv` | Scatter read |
| 20 | `writev` | Gather write |
| 21 | `access` | Check file permissions |
| 22 | `pipe` | Create pipe |
| 23 | `select` | Synchronous I/O multiplexing |
| 24 | `sched_yield` | Yield CPU |
| 25 | `mremap` | Remap memory |
| 26 | `msync` | Sync memory mapping |
| 27 | `mincore` | Check memory residency |
| 28 | `madvise` | Memory usage advice |
| 29 | `shmget` | Get shared memory |
| 30 | `shmat` | Attach shared memory |
| 31 | `shmctl` | Control shared memory |
| 32 | `dup` | Duplicate file descriptor |
| 33 | `dup2` | Duplicate fd to specified fd |

### 2.2 Process System Calls

| Number | System Call | Description |
|--------|-------------|-------------|
| 34 | `pause` | Wait for any signal |
| 35 | `nanosleep` | High-resolution sleep |
| 36 | `getitimer` | Get interval timer |
| 37 | `alarm` | Set timer (SIGALRM) |
| 38 | `setitimer` | Set interval timer |
| 39 | `getpid` | Get process ID |
| 40 | `sendfile` | Send file |
| 41 | `socket` | Create socket |
| 42 | `connect` | Connect |
| 43 | `accept` | Accept connection |
| 44 | `sendto` | Send to address |
| 45 | `recvfrom` | Receive from address |
| 46 | `sendmsg` | Send message |
| 47 | `recvmsg` | Receive message |
| 48 | `shutdown` | Shut down connection |
| 49 | `bind` | Bind address |
| 50 | `listen` | Listen for connections |
| 51 | `getsockname` | Get socket address |
| 52 | `getpeername` | Get peer address |
| 53 | `socketpair` | Create socket pair |
| 54 | `setsockopt` | Set socket option |
| 55 | `getsockopt` | Get socket option |
| 56 | `clone` | Create process/thread |
| 57 | `fork` | Create child process |
| 58 | `vfork` | Create child process (shared address space) |
| 59 | `execve` | Execute program |
| 60 | `exit` | Terminate process |
| 61 | `wait4` | Wait for child process |
| 62 | `kill` | Send signal |
| 63 | `uname` | Get system information |

### 2.3 IPC System Calls

| Number | System Call | Description |
|--------|-------------|-------------|
| 64 | `semget` | Get semaphore |
| 65 | `semop` | Semaphore operations |
| 66 | `semctl` | Control semaphore |
| 67 | `shmdt` | Detach shared memory |
| 68 | `msgget` | Get message queue |
| 69 | `msgsnd` | Send message |
| 70 | `msgrcv` | Receive message |
| 71 | `msgctl` | Control message queue |
| 72 | `fcntl` | File control |
| 73 | `flock` | File lock |
| 74 | `fsync` | Sync file to disk |
| 75 | `fdatasync` | Sync data to disk |

### 2.4 Filesystem System Calls

| Number | System Call | Description |
|--------|-------------|-------------|
| 76 | `truncate` | Truncate file |
| 77 | `ftruncate` | Truncate file by fd |
| 78 | `getdents` | Read directory entries |
| 79 | `getcwd` | Get current working directory |
| 80 | `chdir` | Change working directory |
| 81 | `fchdir` | Change working directory by fd |
| 82 | `rename` | Rename file |
| 83 | `mkdir` | Create directory |
| 84 | `rmdir` | Remove directory |
| 85 | `creat` | Create file |
| 86 | `link` | Create hard link |
| 87 | `unlink` | Delete file |
| 88 | `symlink` | Create symbolic link |
| 89 | `readlink` | Read symbolic link |
| 90 | `chmod` | Change file permissions |
| 91 | `fchmod` | Change file permissions by fd |
| 92 | `chown` | Change file owner |
| 93 | `fchown` | Change file owner by fd |

### 2.5 User/Group/Scheduling System Calls

| Number | System Call | Description |
|--------|-------------|-------------|
| 101 | `getuid` | Get user ID |
| 102 | `getgid` | Get group ID |
| 103 | `setuid` | Set user ID |
| 104 | `setgid` | Set group ID |
| 105 | `geteuid` | Get effective user ID |
| 106 | `getegid` | Get effective group ID |
| 107 | `setpgid` | Set process group ID |
| 108 | `getppid` | Get parent process ID |
| 109 | `getpgrp` | Get process group ID |
| 110 | `setsid` | Create new session |
| 119 | `getpgid` | Get process group ID |

### 2.6 Memory System Calls

| Number | System Call | Description |
|--------|-------------|-------------|
| 9 | `mmap` | Map memory |
| 10 | `mprotect` | Set memory protection |
| 11 | `munmap` | Unmap memory |
| 12 | `brk` | Change data segment size |
| 25 | `mremap` | Remap memory |
| 26 | `msync` | Sync memory mapping |
| 27 | `mincore` | Check memory residency |
| 28 | `madvise` | Memory usage advice |

### 2.7 Signal System Calls

| Number | System Call | Description |
|--------|-------------|-------------|
| 13 | `rt_sigaction` | Real-time signal action setup |
| 14 | `rt_sigprocmask` | Real-time signal mask setup |
| 15 | `rt_sigreturn` | Real-time signal return |
| 125 | `rt_sigpending` | Real-time pending signal query |
| 126 | `rt_sigtimedwait` | Timed wait for real-time signal |
| 127 | `rt_sigqueueinfo` | Real-time signal queued send |

---

## 3. File Operations

### 3.1 open - Open file

```rust
pub fn sys_open(filename: u64, flags: u64, mode: u64) -> Result<u64>
```

**Parameters**:
- `filename`: File path
- `flags`: Open flags (O_RDONLY, O_WRONLY, O_RDWR, O_CREAT, O_TRUNC, etc.)
- `mode`: File permissions (when creating)

**Returns**: File descriptor on success, error code on failure

### 3.2 close - Close file

```rust
pub fn sys_close(fd: u64) -> Result<u64>
```

**Parameters**:
- `fd`: File descriptor

**Returns**: 0 on success, error code on failure

### 3.3 read - Read from file

```rust
pub fn sys_read(fd: u64, buf: u64, count: u64) -> Result<u64>
```

**Parameters**:
- `fd`: File descriptor
- `buf`: Buffer address
- `count`: Number of bytes to read

**Returns**: Number of bytes read, or error code

### 3.4 write - Write to file

```rust
pub fn sys_write(fd: u64, buf: u64, count: u64) -> Result<u64>
```

**Parameters**:
- `fd`: File descriptor
- `buf`: Buffer address
- `count`: Number of bytes to write

**Returns**: Number of bytes written, or error code

### 3.5 lseek - Reposition file offset

```rust
pub fn sys_lseek(fd: u64, offset: u64, whence: u64) -> Result<u64>
```

**Parameters**:
- `fd`: File descriptor
- `offset`: Byte offset
- `whence`: SEEK_SET, SEEK_CUR, or SEEK_END

**Returns**: New offset position

### 3.6 stat - Get file status

```rust
pub fn sys_stat(filename: u64, statbuf: u64) -> Result<u64>
```

### 3.7 fstat - Get file status by descriptor

```rust
pub fn sys_fstat(fd: u64, statbuf: u64) -> Result<u64>
```

### 3.8 Other File Operations

| System Call | Description |
|-------------|-------------|
| `ioctl` | Control device |
| `fcntl` | File control operations |
| `fsync` | Synchronize file with storage |
| `dup` | Duplicate file descriptor |
| `dup2` | Duplicate file descriptor to specified fd |
| `select` | Synchronous I/O multiplexing |
| `poll` | Wait for events on file descriptors |

---

## 4. Process Operations

### 4.1 fork - Create child process

```rust
pub fn sys_fork() -> Result<u64>
```

**Returns**: Parent returns child PID, child returns 0, error code on failure

### 4.2 execve - Execute program

```rust
pub fn sys_execve(filename: u64, argv: u64, envp: u64) -> Result<u64>
```

**Parameters**:
- `filename`: Executable file path
- `argv`: Argument vector
- `envp`: Environment variable vector

**Returns**: Does not return on success, error code on failure

### 4.3 exit - Terminate process

```rust
pub fn sys_exit(error_code: u64) -> Result<u64>
```

**Parameters**:
- `error_code`: Exit status

**Returns**: Does not return

### 4.4 wait4 - Wait for child process

```rust
pub fn sys_wait4(pid: u64, wstatus: u64, options: u64, rusage: u64) -> Result<u64>
```

**Parameters**:
- `pid`: Process ID to wait for
- `wstatus`: Status buffer
- `options`: WNOHANG, WUNTRACED, WCONTINUED
- `rusage`: Resource usage buffer

**Returns**: PID of child whose status changed

### 4.5 kill - Send signal

```rust
pub fn sys_kill(pid: u64, sig: u64) -> Result<u64>
```

**Parameters**:
- `pid`: Process ID
- `sig`: Signal number

**Returns**: 0 on success, error code on failure

### 4.6 Other Process Operations

| System Call | Description |
|-------------|-------------|
| `getpid` | Get process ID |
| `getppid` | Get parent process ID |
| `gettid` | Get thread ID |
| `getpgid` | Get process group ID |
| `setpgid` | Set process group ID |
| `setsid` | Create new session |
| `getuid` | Get user ID |
| `getgid` | Get group ID |
| `geteuid` | Get effective user ID |
| `getegid` | Get effective group ID |
| `sched_yield` | Yield CPU |

---

## 5. Memory Operations

### 5.1 mmap - Map memory

```rust
pub fn sys_mmap(addr: u64, length: u64, prot: u64, flags: u64, fd: u64, offset: u64) -> Result<u64>
```

**Parameters**:
- `addr`: Suggested address
- `length`: Mapping length
- `prot`: Protection flags (PROT_READ, PROT_WRITE, PROT_EXEC)
- `flags`: Mapping flags (MAP_PRIVATE, MAP_SHARED, MAP_ANONYMOUS)
- `fd`: File descriptor (for file mapping)
- `offset`: Offset within file

**Returns**: Mapped address

### 5.2 munmap - Unmap memory

```rust
pub fn sys_munmap(addr: u64, length: u64) -> Result<u64>
```

### 5.3 mprotect - Change memory protection

```rust
pub fn sys_mprotect(addr: u64, len: u64, prot: u64) -> Result<u64>
```

### 5.4 brk - Change data segment size

```rust
pub fn sys_brk(addr: u64) -> Result<u64>
```

---

## 6. IPC Operations

### 6.1 pipe - Create pipe

```rust
pub fn sys_pipe(fds: u64) -> Result<u64>
```

### 6.2 shmget - Get shared memory

```rust
pub fn sys_shmget(key: u64, size: u64, flags: u64) -> Result<u64>
```

### 6.3 shmat - Attach shared memory

```rust
pub fn sys_shmat(shmid: u64, shmaddr: u64, shmflg: u64) -> Result<u64>
```

### 6.4 shmdt - Detach shared memory

```rust
pub fn sys_shmdt(shmaddr: u64) -> Result<u64>
```

### 6.5 semget - Get semaphore

```rust
pub fn sys_semget(key: u64, nsems: u64, flags: u64) -> Result<u64>
```

### 6.6 semop - Semaphore operations

```rust
pub fn sys_semop(semid: u64, sops: u64, nsops: u64) -> Result<u64>
```

### 6.7 msgget - Get message queue

```rust
pub fn sys_msgget(key: u64, flags: u64) -> Result<u64>
```

### 6.8 msgsnd - Send message

```rust
pub fn sys_msgsnd(msqid: u64, msgp: u64, msgsz: u64, msgflg: u64) -> Result<u64>
```

### 6.9 msgrcv - Receive message

```rust
pub fn sys_msgrcv(msqid: u64, msgp: u64, msgsz: u64, msgtyp: u64, msgflg: u64) -> Result<u64>
```

---

## 7. Network Operations

### 7.1 socket - Create socket

```rust
pub fn sys_socket(domain: u64, sock_type: u64, protocol: u64) -> Result<u64>
```

### 7.2 bind - Bind socket address

```rust
pub fn sys_bind(sockfd: u64, addr: u64, addrlen: u64) -> Result<u64>
```

### 7.3 listen - Listen for connections

```rust
pub fn sys_listen(sockfd: u64, backlog: u64) -> Result<u64>
```

### 7.4 accept - Accept connection

```rust
pub fn sys_accept(sockfd: u64, addr: u64, addrlen: u64) -> Result<u64>
```

### 7.5 connect - Connect socket

```rust
pub fn sys_connect(sockfd: u64, addr: u64, addrlen: u64) -> Result<u64>
```

### 7.6 send - Send data

```rust
pub fn sys_send(sockfd: u64, buf: u64, len: u64, flags: u64) -> Result<u64>
```

### 7.7 recv - Receive data

```rust
pub fn sys_recv(sockfd: u64, buf: u64, len: u64, flags: u64) -> Result<u64>
```

---

## 8. Signal Handling System Calls

### 8.1 sigaction - Set signal handler

```rust
pub fn sys_sigaction(sig: u64, act: u64, oact: u64) -> Result<u64>
```

**Parameters**:
- `sig`: Signal number (1-31 standard, 32-64 real-time)
- `act`: New signal action structure (`sigaction` struct)
- `oact`: Previous signal action structure (output)

**sigaction structure**:
```c
struct sigaction {
    void (*sa_handler)(int);     // Handler function
    sigset_t sa_mask;            // Signals blocked during handling
    int sa_flags;                // Flags (SA_RESTART, SA_SIGINFO, etc.)
};
```

### 8.2 sigprocmask - Set signal mask

```rust
pub fn sys_sigprocmask(how: u64, set: u64, oset: u64) -> Result<u64>
```

**how parameter**:
- `SIG_BLOCK`: Add signals in set to blocked set
- `SIG_UNBLOCK`: Remove signals in set from blocked set
- `SIG_SETMASK`: Set blocked set to set

### 8.3 sigpending - Get pending signals

```rust
pub fn sys_sigpending(set: u64) -> Result<u64>
```

### 8.4 sigsuspend - Wait for signal

```rust
pub fn sys_sigsuspend(mask: u64) -> Result<u64>
```

Atomically replaces signal mask and suspends process, waiting for signal.

### 8.5 Other Signal System Calls

| System Call | Description |
|-------------|-------------|
| `kill(pid, sig)` | Send signal to process |
| `tgkill(tgid, tid, sig)` | Send signal to thread |
| `raise(sig)` | Send signal to self |
| `pause()` | Wait for any signal |
| `alarm(seconds)` | Set SIGALRM timer |
| `rt_sigaction` | Real-time signal action setup |
| `rt_sigprocmask` | Real-time signal mask setup |
| `rt_sigpending` | Real-time pending signal query |
| `rt_sigsuspend` | Real-time signal wait |
| `rt_sigqueueinfo` | Real-time signal queued send |
| `rt_sigtimedwait` | Timed wait for real-time signal |

---

## 9. io_uring System Calls

### 9.1 io_uring_setup - Create io_uring instance

```rust
pub fn sys_io_uring_setup(entries: u64, params: u64) -> Result<u64>
```

**Parameters**:
- `entries`: Submission queue depth (must be power of 2)
- `params`: `io_uring_params` structure

**Returns**: io_uring file descriptor

**io_uring_params structure**:
```c
struct io_uring_params {
    u32 sq_entries;          // Actual SQ entry count
    u32 cq_entries;          // Actual CQ entry count
    u32 flags;               // Feature flags
    u32 sq_off;              // SQ offset
    u32 cq_off;              // CQ offset
    // ...
};
```

### 9.2 io_uring_enter - Submit and wait for IO

```rust
pub fn sys_io_uring_enter(fd: u64, to_submit: u64, min_complete: u64, flags: u64, sig: u64, sz: u64) -> Result<u64>
```

**Parameters**:
- `fd`: io_uring file descriptor
- `to_submit`: Number of SQEs to submit
- `min_complete`: Minimum completions to wait for
- `flags`: `IORING_ENTER_GETEVENTS`, `IORING_ENTER_SQ_WAKEUP`

**Returns**: Number of completed CQEs

### 9.3 io_uring_register - Register resources

```rust
pub fn sys_io_uring_register(fd: u64, opcode: u64, arg: u64, nr_args: u64) -> Result<u64>
```

**opcode**:
- `IORING_REGISTER_BUFFERS`: Register fixed buffers
- `IORING_UNREGISTER_BUFFERS`: Unregister buffers
- `IORING_REGISTER_FILES`: Register file descriptors
- `IORING_UNREGISTER_FILES`: Unregister files
- `IORING_REGISTER_EVENTFD`: Register event notification fd
- `IORING_REGISTER_PROBE`: Query opcode support

---

## 10. Error Codes

### 10.1 POSIX Error Codes

```rust
pub enum ErrorCode {
    Success = 0,
    EPERM = 1,           // Operation not permitted
    ENOENT = 2,          // No such file or directory
    ESRCH = 3,           // No such process
    EINTR = 4,           // Interrupted system call
    EIO = 5,             // I/O error
    ENXIO = 6,           // No such device or address
    E2BIG = 7,           // Argument list too long
    ENOEXEC = 8,         // Exec format error
    EBADF = 9,           // Bad file number
    ECHILD = 10,         // No child processes
    EAGAIN = 11,         // Try again
    ENOMEM = 12,         // Out of memory
    EACCES = 13,         // Permission denied
    EFAULT = 14,         // Bad address
    ENOTBLK = 15,        // Block device required
    EBUSY = 16,          // Device or resource busy
    EEXIST = 17,         // File exists
    EXDEV = 18,          // Cross-device link
    ENODEV = 19,         // No such device
    ENOTDIR = 20,        // Not a directory
    EISDIR = 21,         // Is a directory
    EINVAL = 22,         // Invalid argument
    ENFILE = 23,         // File table overflow
    EMFILE = 24,         // Too many open files
    ENOTTY = 25,         // Not a typewriter
    ETXTBSY = 26,        // Text file busy
    EFBIG = 27,          // File too large
    ENOSPC = 28,         // No space left on device
    ESPIPE = 29,         // Illegal seek
    EROFS = 30,          // Read-only file system
    EMLINK = 31,         // Too many links
    EPIPE = 32,          // Broken pipe
    EDOM = 33,           // Math argument out of domain of func
    ERANGE = 34,         // Math result not representable
    EDEADLK = 35,        // Resource deadlock would occur
    ENAMETOOLONG = 36,   // File name too long
    ENOLCK = 37,         // No record locks available
    ENOSYS = 38,         // Function not implemented
    ENOTEMPTY = 39,      // Directory not empty
    ELOOP = 40,          // Too many symbolic links encountered
    EWOULDBLOCK = 41,    // Operation would block
    ENOMSG = 42,         // No message of desired type
    EIDRM = 43,          // Identifier removed
    ECHRNG = 44,         // Channel number out of range
    EL2NSYNC = 45,       // Level 2 not synchronized
    EL3HLT = 46,         // Level 3 halted
    EL3RST = 47,         // Level 3 reset
    ELNRNG = 48,         // Link number out of range
    EUNATCH = 49,        // Protocol driver not attached
    ENOCSI = 50,         // No CSI structure available
    EL2HLT = 51,         // Level 2 halted
    EBADE = 52,          // Invalid exchange
    EBADR = 53,          // Invalid request descriptor
    EXFULL = 54,         // Exchange full
    ENOANO = 55,         // No anode
    EBADRPC = 56,        // RPC struct is bad
    ERPCMACH = 57,       // RPC version wrong
    ERPCNOTFOUND = 58,   // RPC program not found
    EPROTONOSUPPORT = 59, // Protocol not supported
    ESOCKTNOSUPPORT = 60, // Socket type not supported
    EOPNOTSUPP = 61,     // Operation not supported
    EPFNOSUPPORT = 62,   // Protocol family not supported
    EAFNOSUPPORT = 63,   // Address family not supported
    EADDRINUSE = 64,     // Address already in use
    EADDRNOTAVAIL = 65,  // Address not available
    ENETDOWN = 66,       // Network is down
    ENETUNREACH = 67,    // Network is unreachable
    ENETRESET = 68,      // Network dropped connection
    ECONNABORTED = 69,   // Software caused connection abort
    ECONNRESET = 70,     // Connection reset by peer
    ENOBUFS = 71,        // No buffer space available
    EISCONN = 72,        // Transport endpoint is already connected
    ENOTCONN = 73,       // Transport endpoint is not connected
    ESHUTDOWN = 74,      // Cannot send after transport endpoint shutdown
    ETIMEDOUT = 75,      // Connection timed out
    ECONNREFUSED = 76,   // Connection refused
    EHOSTDOWN = 77,      // Host is down
    EHOSTUNREACH = 78,   // No route to host
    EALREADY = 79,       // Operation already in progress
    EINPROGRESS = 80,    // Operation now in progress
    ESTALE = 81,         // Stale file handle
    EUCLEAN = 82,        // Structure needs cleaning
    ENOTNAM = 83,        // Not a XENIX named type file
    ENAVAIL = 84,        // No XENIX semaphores available
    EISNAM = 85,         // Is a named type file
    EREMOTEIO = 86,      // Remote I/O error
    EDQUOT = 87,         // Quota exceeded
    ENOMEDIUM = 88,      // No medium found
    EMEDIUMTYPE = 89,    // Wrong medium type
    ECANCELED = 90,      // Operation canceled
    ENOKEY = 91,         // Required key not available
    EKEYEXPIRED = 92,    // Key has expired
    EKEYREVOKED = 93,    // Key has been revoked
    EKEYREJECTED = 94,   // Key was rejected by service
}
```

### 10.2 Error Handling

System calls return error codes as negative values:

```rust
pub type Result<T> = core::result::Result<T, ErrorCode>;

pub fn sys_read(fd: u64, buf: u64, count: u64) -> Result<u64> {
    // ... implementation
    Ok(bytes_read)  // Success
    // or
    Err(ErrorCode::EBADF)  // Error
}
```

---

## 11. File Structure

```
kernel/syscall/
├── mod.rs              # System call module
├── handler.rs          # System call handler and number table
├── impl.rs             # System call implementations
├── file.rs             # File operations
├── process.rs          # Process operations
├── process_integration.rs # Process integration
└── error.rs            # Error codes (in kernel module)
```

---

**Last Updated**: May 15, 2026
**License**: Apache-2.0
