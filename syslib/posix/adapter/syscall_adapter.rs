/*
 * Nuva OS - Syslib - POSIX Syscall Adapter
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

use crate::kernel::posix::Syscall;
use crate::kernel::ipc::nuvaipc::MessageBits;
use super::errno_adapter::ErrnoAdapter;

/// POSIX syscall request adapter
/// Translates POSIX system call requests into kernel IPC messages.
pub struct SyscallAdapter;

impl SyscallAdapter {
    /// Translate a POSIX syscall number and arguments into an IPC message
    pub fn translate_request(
        syscall: Syscall,
        args: [u64; 6],
    ) -> IpcRequest {
        IpcRequest {
            syscall_num: syscall as u32,
            args,
            flags: MessageBits::empty(),
        }
    }

    /// Dispatch open() through IPC
    /// semantics: O_CREAT|O_EXCL → EEXIST on existing file.
    pub fn dispatch_open(path: &str, flags: u32, mode: u32) -> IpcRequest {
        let args = [
            path.as_ptr() as u64,
            flags as u64,
            mode as u64,
            0, 0, 0,
        ];
        Self::translate_request(Syscall::Open, args)
    }

    /// Dispatch read() through IPC
    /// semantics: Returns EAGAIN for O_NONBLOCK if no data.
    pub fn dispatch_read(fd: i32, count: usize) -> IpcRequest {
        let args = [fd as u64, 0, count as u64, 0, 0, 0];
        Self::translate_request(Syscall::Read, args)
    }

    /// Dispatch write() through IPC
    /// semantics: Returns EAGAIN for O_NONBLOCK if no space.
    /// Returns EPIPE if fd is connected to a pipe/sock whose read end is closed.
    pub fn dispatch_write(fd: i32, count: usize) -> IpcRequest {
        let args = [fd as u64, 0, count as u64, 0, 0, 0];
        Self::translate_request(Syscall::Write, args)
    }

    /// Dispatch fork() through IPC
    /// semantics: Child gets PID 0, parent gets child PID.
    /// Returns ENOMEM if insufficient kernel resources.
    pub fn dispatch_fork() -> IpcRequest {
        Self::translate_request(Syscall::Fork, [0; 6])
    }

    /// Dispatch execve() through IPC
    /// semantics: Replaces process image. Returns ENOENT if path not found.
    /// Returns EACCES if execute permission denied. Returns ENOEXEC if not executable format.
    pub fn dispatch_execve(path: &str) -> IpcRequest {
        let args = [path.as_ptr() as u64, 0, 0, 0, 0, 0];
        Self::translate_request(Syscall::Execve, args)
    }

    /// Dispatch kill() through IPC
    /// semantics: ESRCH if pid not found, EPERM if no permission.
    pub fn dispatch_kill(pid: i32, sig: i32) -> IpcRequest {
        let args = [pid as u64, sig as u64, 0, 0, 0, 0];
        Self::translate_request(Syscall::Kill, args)
    }

    /// Dispatch mmap() through IPC
    /// semantics: Returns EINVAL for invalid parameters.
    /// Returns ENOMEM if mapping cannot be established.
    pub fn dispatch_mmap(addr: u64, len: usize, prot: i32, flags: i32, fd: i32, offset: i64) -> IpcRequest {
        let args = [addr, len as u64, prot as u64, flags as u64, fd as u64, offset as u64];
        Self::translate_request(Syscall::Mmap, args)
    }

    /// Dispatch socket() through IPC
    /// semantics: Returns EAFNOSUPPORT if address family not supported.
    /// Returns EPROTONOSUPPORT if protocol not supported.
    /// POSIX DEVIATION: Partial socket support. AF_UNIX/AF_INET only.
    /// AF_INET6, AF_NETLINK return EAFNOSUPPORT. Raw sockets return EPROTONOSUPPORT.
    pub fn dispatch_socket(domain: i32, type_: i32, protocol: i32) -> IpcRequest {
        let args = [domain as u64, type_ as u64, protocol as u64, 0, 0, 0];
        Self::translate_request(Syscall::Socket, args)
    }

    /// Translate IPC response back to POSIX result
    pub fn translate_response(response: IpcResponse) -> Result<i64, i32> {
        if response.error_code == 0 {
            Ok(response.return_value)
        } else {
            let errno = ErrnoAdapter::from_kernel(response.error_code);
            Err(-(errno as i32))
        }
    }
}

/// IPC request structure (POSIX → kernel)
#[derive(Debug, Clone, Copy)]
pub struct IpcRequest {
    pub syscall_num: u32,
    pub args: [u64; 6],
    pub flags: MessageBits,
}

/// IPC response structure (kernel → POSIX)
#[derive(Debug, Clone, Copy)]
pub struct IpcResponse {
    pub return_value: i64,
    pub error_code: i32,
}
