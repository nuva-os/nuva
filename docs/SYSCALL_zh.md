# Nuva OS 系统调用模块

## 概述

系统调用模块提供与 POSIX 兼容的接口，供用户空间应用程序与 kernel 交互。包含完整的系统调用号表、错误码定义、信号处理和 io_uring 相关系统调用。

---

## 目录

1. [系统调用架构](#1-系统调用架构)
2. [系统调用号表](#2-系统调用号表)
3. [文件操作](#3-文件操作)
4. [进程操作](#4-进程操作)
5. [内存操作](#5-内存操作)
6. [IPC 操作](#6-ipc-操作)
7. [网络操作](#7-网络操作)
8. [信号处理系统调用](#8-信号处理系统调用)
9. [io_uring 系统调用](#9-io_uring-系统调用)
10. [错误码](#10-错误码)
11. [文件结构](#11-文件结构)

---

## 1. 系统调用架构

### 1.1 系统调用接口

系统调用是用户空间应用程序与 kernel 之间的主要接口。它们提供对 kernel 服务的受控访问。

### 1.2 系统调用约定

```rust
pub type SyscallHandler = fn(u64, u64, u64, u64, u64) -> Result<u64>;
```

系统调用最多接受 6 个参数（包括系统调用号）并返回一个结果。

### 1.3 系统调用表

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

## 2. 系统调用号表

### 2.1 文件 I/O 系统调用

| 号 | 系统调用 | 说明 |
|----|----------|------|
| 0 | `read` | 从文件描述符读取 |
| 1 | `write` | 向文件描述符写入 |
| 2 | `open` | 打开文件 |
| 3 | `close` | 关闭文件描述符 |
| 4 | `stat` | 获取文件状态 |
| 5 | `fstat` | 通过 fd 获取文件状态 |
| 6 | `lstat` | 获取符号链接文件状态 |
| 7 | `poll` | 等待文件描述符事件 |
| 8 | `lseek` | 设置文件偏移 |
| 9 | `mmap` | 映射内存 |
| 10 | `mprotect` | 设置内存保护 |
| 11 | `munmap` | 取消内存映射 |
| 12 | `brk` | 更改数据段大小 |
| 13 | `rt_sigaction` | 实时信号处理设置 |
| 14 | `rt_sigprocmask` | 实时信号掩码设置 |
| 15 | `rt_sigreturn` | 实时信号返回 |
| 16 | `ioctl` | 设备控制 |
| 17 | `pread64` | 从指定偏移读取 |
| 18 | `pwrite64` | 从指定偏移写入 |
| 19 | `readv` | 向量读取 |
| 20 | `writev` | 向量写入 |
| 21 | `access` | 检查文件权限 |
| 22 | `pipe` | 创建管道 |
| 23 | `select` | 同步 I/O 多路复用 |
| 24 | `sched_yield` | 让出 CPU |
| 25 | `mremap` | 重新映射内存 |
| 26 | `msync` | 同步内存映射 |
| 27 | `mincore` | 检查内存驻留 |
| 28 | `madvise` | 内存使用建议 |
| 29 | `shmget` | 获取共享内存 |
| 30 | `shmat` | 附接共享内存 |
| 31 | `shmctl` | 控制共享内存 |
| 32 | `dup` | 复制文件描述符 |
| 33 | `dup2` | 复制 fd 到指定 fd |

### 2.2 进程系统调用

| 号 | 系统调用 | 说明 |
|----|----------|------|
| 34 | `pause` | 等待任意信号 |
| 35 | `nanosleep` | 高精度睡眠 |
| 36 | `getitimer` | 获取定时器 |
| 37 | `alarm` | 设置定时器（SIGALRM） |
| 38 | `setitimer` | 设置定时器 |
| 39 | `getpid` | 获取进程 ID |
| 40 | `sendfile` | 发送文件 |
| 41 | `socket` | 创建套接字 |
| 42 | `connect` | 连接 |
| 43 | `accept` | 接受连接 |
| 44 | `sendto` | 发送到指定地址 |
| 45 | `recvfrom` | 从指定地址接收 |
| 46 | `sendmsg` | 发送消息 |
| 47 | `recvmsg` | 接收消息 |
| 48 | `shutdown` | 关闭连接 |
| 49 | `bind` | 绑定地址 |
| 50 | `listen` | 监听连接 |
| 51 | `getsockname` | 获取套接字地址 |
| 52 | `getpeername` | 获取对端地址 |
| 53 | `socketpair` | 创建套接字对 |
| 54 | `setsockopt` | 设置套接字选项 |
| 55 | `getsockopt` | 获取套接字选项 |
| 56 | `clone` | 创建进程/线程 |
| 57 | `fork` | 创建子进程 |
| 58 | `vfork` | 创建子进程（共享地址空间） |
| 59 | `execve` | 执行程序 |
| 60 | `exit` | 终止进程 |
| 61 | `wait4` | 等待子进程 |
| 62 | `kill` | 发送信号 |
| 63 | `uname` | 获取系统信息 |

### 2.3 IPC 系统调用

| 号 | 系统调用 | 说明 |
|----|----------|------|
| 64 | `semget` | 获取信号量 |
| 65 | `semop` | 信号量操作 |
| 66 | `semctl` | 控制信号量 |
| 67 | `shmdt` | 分离共享内存 |
| 68 | `msgget` | 获取消息队列 |
| 69 | `msgsnd` | 发送消息 |
| 70 | `msgrcv` | 接收消息 |
| 71 | `msgctl` | 控制消息队列 |
| 72 | `fcntl` | 文件控制 |
| 73 | `flock` | 文件锁 |
| 74 | `fsync` | 同步文件到磁盘 |
| 75 | `fdatasync` | 同步数据到磁盘 |

### 2.4 文件系统系统调用

| 号 | 系统调用 | 说明 |
|----|----------|------|
| 76 | `truncate` | 截断文件 |
| 77 | `ftruncate` | 通过 fd 截断文件 |
| 78 | `getdents` | 读取目录项 |
| 79 | `getcwd` | 获取当前工作目录 |
| 80 | `chdir` | 改变工作目录 |
| 81 | `fchdir` | 通过 fd 改变工作目录 |
| 82 | `rename` | 重命名文件 |
| 83 | `mkdir` | 创建目录 |
| 84 | `rmdir` | 删除目录 |
| 85 | `creat` | 创建文件 |
| 86 | `link` | 创建硬链接 |
| 87 | `unlink` | 删除文件 |
| 88 | `symlink` | 创建符号链接 |
| 89 | `readlink` | 读取符号链接 |
| 90 | `chmod` | 更改文件权限 |
| 91 | `fchmod` | 通过 fd 更改文件权限 |
| 92 | `chown` | 更改文件所有者 |
| 93 | `fchown` | 通过 fd 更改文件所有者 |

### 2.5 用户/组/调度系统调用

| 号 | 系统调用 | 说明 |
|----|----------|------|
| 101 | `getuid` | 获取用户 ID |
| 102 | `getgid` | 获取组 ID |
| 103 | `setuid` | 设置用户 ID |
| 104 | `setgid` | 设置组 ID |
| 105 | `geteuid` | 获取有效用户 ID |
| 106 | `getegid` | 获取有效组 ID |
| 107 | `setpgid` | 设置进程组 ID |
| 108 | `getppid` | 获取父进程 ID |
| 109 | `getpgrp` | 获取进程组 ID |
| 110 | `setsid` | 创建新会话 |
| 119 | `getpgid` | 获取进程组 ID |

### 2.6 内存系统调用

| 号 | 系统调用 | 说明 |
|----|----------|------|
| 9 | `mmap` | 映射内存 |
| 10 | `mprotect` | 设置内存保护 |
| 11 | `munmap` | 取消内存映射 |
| 12 | `brk` | 更改数据段大小 |
| 25 | `mremap` | 重新映射内存 |
| 26 | `msync` | 同步内存映射 |
| 27 | `mincore` | 检查内存驻留 |
| 28 | `madvise` | 内存使用建议 |

### 2.7 信号系统调用

| 号 | 系统调用 | 说明 |
|----|----------|------|
| 13 | `rt_sigaction` | 实时信号处理设置 |
| 14 | `rt_sigprocmask` | 实时信号掩码设置 |
| 15 | `rt_sigreturn` | 实时信号返回 |
| 125 | `rt_sigpending` | 实时挂起信号查询 |
| 126 | `rt_sigtimedwait` | 定时等待实时信号 |
| 127 | `rt_sigqueueinfo` | 实时信号排队发送 |

---

## 3. 文件操作

### 3.1 open - 打开文件

```rust
pub fn sys_open(filename: u64, flags: u64, mode: u64) -> Result<u64>
```

**参数**：
- `filename`：文件路径
- `flags`：打开标志（O_RDONLY、O_WRONLY、O_RDWR、O_CREAT、O_TRUNC 等）
- `mode`：文件权限（创建时）

**返回**：成功时返回文件描述符，失败时返回错误码

### 3.2 close - 关闭文件

```rust
pub fn sys_close(fd: u64) -> Result<u64>
```

**参数**：
- `fd`：文件描述符

**返回**：成功时返回 0，失败时返回错误码

### 3.3 read - 从文件读取

```rust
pub fn sys_read(fd: u64, buf: u64, count: u64) -> Result<u64>
```

**参数**：
- `fd`：文件描述符
- `buf`：缓冲区地址
- `count`：要读取的字节数

**返回**：读取的字节数，或错误码

### 3.4 write - 写入文件

```rust
pub fn sys_write(fd: u64, buf: u64, count: u64) -> Result<u64>
```

**参数**：
- `fd`：文件描述符
- `buf`：缓冲区地址
- `count`：要写入的字节数

**返回**：写入的字节数，或错误码

### 3.5 lseek - 重新定位文件偏移

```rust
pub fn sys_lseek(fd: u64, offset: u64, whence: u64) -> Result<u64>
```

**参数**：
- `fd`：文件描述符
- `offset`：偏移字节数
- `whence`：SEEK_SET、SEEK_CUR 或 SEEK_END

**返回**：新的偏移位置

### 3.6 stat - 获取文件状态

```rust
pub fn sys_stat(filename: u64, statbuf: u64) -> Result<u64>
```

### 3.7 fstat - 通过描述符获取文件状态

```rust
pub fn sys_fstat(fd: u64, statbuf: u64) -> Result<u64>
```

### 3.8 其他文件操作

| 系统调用 | 描述 |
|-------------|-------------|
| `ioctl` | 控制设备 |
| `fcntl` | 文件控制操作 |
| `fsync` | 将文件与存储同步 |
| `dup` | 复制文件描述符 |
| `dup2` | 复制文件描述符到指定 fd |
| `select` | 同步 I/O 多路复用 |
| `poll` | 等待文件描述符上的事件 |

---

## 4. 进程操作

### 4.1 fork - 创建子进程

```rust
pub fn sys_fork() -> Result<u64>
```

**返回**：父进程返回子进程 PID，子进程返回 0，失败时返回错误码

### 4.2 execve - 执行程序

```rust
pub fn sys_execve(filename: u64, argv: u64, envp: u64) -> Result<u64>
```

**参数**：
- `filename`：可执行文件路径
- `argv`：参数向量
- `envp`：环境变量向量

**返回**：成功时不返回，失败时返回错误码

### 4.3 exit - 终止进程

```rust
pub fn sys_exit(error_code: u64) -> Result<u64>
```

**参数**：
- `error_code`：退出状态

**返回**：不返回

### 4.4 wait4 - 等待子进程

```rust
pub fn sys_wait4(pid: u64, wstatus: u64, options: u64, rusage: u64) -> Result<u64>
```

**参数**：
- `pid`：要等待的进程 ID
- `wstatus`：状态缓冲区
- `options`：WNOHANG、WUNTRACED、WCONTINUED
- `rusage`：资源使用缓冲区

**返回**：状态发生变化的子进程 PID

### 4.5 kill - 发送信号

```rust
pub fn sys_kill(pid: u64, sig: u64) -> Result<u64>
```

**参数**：
- `pid`：进程 ID
- `sig`：信号编号

**返回**：成功时返回 0，失败时返回错误码

### 4.6 其他进程操作

| 系统调用 | 描述 |
|-------------|-------------|
| `getpid` | 获取进程 ID |
| `getppid` | 获取父进程 ID |
| `gettid` | 获取线程 ID |
| `getpgid` | 获取进程组 ID |
| `setpgid` | 设置进程组 ID |
| `setsid` | 创建新会话 |
| `getuid` | 获取用户 ID |
| `getgid` | 获取组 ID |
| `geteuid` | 获取有效用户 ID |
| `getegid` | 获取有效组 ID |
| `sched_yield` | 让出 CPU |

---

## 5. 内存操作

### 5.1 mmap - 映射内存

```rust
pub fn sys_mmap(addr: u64, length: u64, prot: u64, flags: u64, fd: u64, offset: u64) -> Result<u64>
```

**参数**：
- `addr`：建议地址
- `length`：映射长度
- `prot`：保护标志（PROT_READ、PROT_WRITE、PROT_EXEC）
- `flags`：映射标志（MAP_PRIVATE、MAP_SHARED、MAP_ANONYMOUS）
- `fd`：文件描述符（用于文件映射）
- `offset`：文件内偏移

**返回**：映射地址

### 5.2 munmap - 取消内存映射

```rust
pub fn sys_munmap(addr: u64, length: u64) -> Result<u64>
```

### 5.3 mprotect - 更改内存保护

```rust
pub fn sys_mprotect(addr: u64, len: u64, prot: u64) -> Result<u64>
```

### 5.4 brk - 更改数据段大小

```rust
pub fn sys_brk(addr: u64) -> Result<u64>
```

---

## 6. IPC 操作

### 6.1 pipe - 创建管道

```rust
pub fn sys_pipe(fds: u64) -> Result<u64>
```

### 6.2 shmget - 获取共享内存

```rust
pub fn sys_shmget(key: u64, size: u64, flags: u64) -> Result<u64>
```

### 6.3 shmat - 附接共享内存

```rust
pub fn sys_shmat(shmid: u64, shmaddr: u64, shmflg: u64) -> Result<u64>
```

### 6.4 shmdt - 分离共享内存

```rust
pub fn sys_shmdt(shmaddr: u64) -> Result<u64>
```

### 6.5 semget - 获取信号量

```rust
pub fn sys_semget(key: u64, nsems: u64, flags: u64) -> Result<u64>
```

### 6.6 semop - 信号量操作

```rust
pub fn sys_semop(semid: u64, sops: u64, nsops: u64) -> Result<u64>
```

### 6.7 msgget - 获取消息队列

```rust
pub fn sys_msgget(key: u64, flags: u64) -> Result<u64>
```

### 6.8 msgsnd - 发送消息

```rust
pub fn sys_msgsnd(msqid: u64, msgp: u64, msgsz: u64, msgflg: u64) -> Result<u64>
```

### 6.9 msgrcv - 接收消息

```rust
pub fn sys_msgrcv(msqid: u64, msgp: u64, msgsz: u64, msgtyp: u64, msgflg: u64) -> Result<u64>
```

---

## 7. 网络操作

### 7.1 socket - 创建套接字

```rust
pub fn sys_socket(domain: u64, sock_type: u64, protocol: u64) -> Result<u64>
```

### 7.2 bind - 绑定套接字地址

```rust
pub fn sys_bind(sockfd: u64, addr: u64, addrlen: u64) -> Result<u64>
```

### 7.3 listen - 监听连接

```rust
pub fn sys_listen(sockfd: u64, backlog: u64) -> Result<u64>
```

### 7.4 accept - 接受连接

```rust
pub fn sys_accept(sockfd: u64, addr: u64, addrlen: u64) -> Result<u64>
```

### 7.5 connect - 连接套接字

```rust
pub fn sys_connect(sockfd: u64, addr: u64, addrlen: u64) -> Result<u64>
```

### 7.6 send - 发送数据

```rust
pub fn sys_send(sockfd: u64, buf: u64, len: u64, flags: u64) -> Result<u64>
```

### 7.7 recv - 接收数据

```rust
pub fn sys_recv(sockfd: u64, buf: u64, len: u64, flags: u64) -> Result<u64>
```

---

## 8. 信号处理系统调用

### 8.1 sigaction - 设置信号处理

```rust
pub fn sys_sigaction(sig: u64, act: u64, oact: u64) -> Result<u64>
```

**参数**：
- `sig`：信号编号（1-31 标准信号，32-64 实时信号）
- `act`：新的信号处理结构（`sigaction` 结构体）
- `oact`：旧的信号处理结构（输出）

**sigaction 结构**：
```c
struct sigaction {
    void (*sa_handler)(int);     // 处理函数
    sigset_t sa_mask;            // 处理期间阻塞的信号集
    int sa_flags;                // 标志（SA_RESTART、SA_SIGINFO 等）
};
```

### 8.2 sigprocmask - 设置信号掩码

```rust
pub fn sys_sigprocmask(how: u64, set: u64, oset: u64) -> Result<u64>
```

**how 参数**：
- `SIG_BLOCK`：将 set 中的信号加入阻塞集
- `SIG_UNBLOCK`：将 set 中的信号从阻塞集移除
- `SIG_SETMASK`：将阻塞集设为 set

### 8.3 sigpending - 获取挂起信号

```rust
pub fn sys_sigpending(set: u64) -> Result<u64>
```

### 8.4 sigsuspend - 等待信号

```rust
pub fn sys_sigsuspend(mask: u64) -> Result<u64>
```

原子替换信号掩码并暂停进程，等待信号。

### 8.5 其他信号系统调用

| 系统调用 | 描述 |
|----------|------|
| `kill(pid, sig)` | 向进程发送信号 |
| `tgkill(tgid, tid, sig)` | 向线程发送信号 |
| `raise(sig)` | 向自身发送信号 |
| `pause()` | 等待任意信号 |
| `alarm(seconds)` | 设置 SIGALRM 定时器 |
| `rt_sigaction` | 实时信号处理设置 |
| `rt_sigprocmask` | 实时信号掩码设置 |
| `rt_sigpending` | 实时挂起信号查询 |
| `rt_sigsuspend` | 实时信号等待 |
| `rt_sigqueueinfo` | 实时信号排队发送 |
| `rt_sigtimedwait` | 定时等待实时信号 |

---

## 9. io_uring 系统调用

### 9.1 io_uring_setup - 创建 io_uring 实例

```rust
pub fn sys_io_uring_setup(entries: u64, params: u64) -> Result<u64>
```

**参数**：
- `entries`：提交队列深度（必须为 2 的幂）
- `params`：`io_uring_params` 结构体

**返回**：io_uring 文件描述符

**io_uring_params 结构**：
```c
struct io_uring_params {
    u32 sq_entries;          // SQ 实际条目数
    u32 cq_entries;          // CQ 实际条目数
    u32 flags;               // 特性标志
    u32 sq_off;              // SQ 偏移
    u32 cq_off;              // CQ 偏移
    // ...
};
```

### 9.2 io_uring_enter - 提交和等待 IO

```rust
pub fn sys_io_uring_enter(fd: u64, to_submit: u64, min_complete: u64, flags: u64, sig: u64, sz: u64) -> Result<u64>
```

**参数**：
- `fd`：io_uring 文件描述符
- `to_submit`：要提交的 SQE 数量
- `min_complete`：最少等待完成数
- `flags`：`IORING_ENTER_GETEVENTS`、`IORING_ENTER_SQ_WAKEUP`

**返回**：完成的 CQE 数量

### 9.3 io_uring_register - 注册资源

```rust
pub fn sys_io_uring_register(fd: u64, opcode: u64, arg: u64, nr_args: u64) -> Result<u64>
```

**opcode**：
- `IORING_REGISTER_BUFFERS`：注册固定缓冲区
- `IORING_UNREGISTER_BUFFERS`：取消注册缓冲区
- `IORING_REGISTER_FILES`：注册文件描述符
- `IORING_UNREGISTER_FILES`：取消注册文件
- `IORING_REGISTER_EVENTFD`：注册事件通知 fd
- `IORING_REGISTER_PROBE`：查询操作码支持

---

## 10. 错误码

### 10.1 POSIX 错误码

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

### 10.2 错误处理

系统调用以负值返回错误码：

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

## 11. 文件结构

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

**最后更新**：2026 年 5 月 15 日
**许可证**：Apache-2.0
