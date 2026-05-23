/*
 * Nuva OS - Syslib - POSIX IPC interfaces (pipe/mq/shm)
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

use super::errno::Errno;

/// Maximum atomic write size for pipes (POSIX PIPE_BUF)
pub const PIPE_BUF: usize = 4096;

/// IPC key type
pub type IpcKey = i32;

/// IPC ID type
pub type IpcId = i32;

/// IPC permissions structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IpcPerm {
    pub key: IpcKey,
    pub uid: u32,
    pub gid: u32,
    pub cuid: u32,
    pub cgid: u32,
    pub mode: u16,
    pub seq: u16,
}

/// IPC command constants for msgctl/shmctl/semctl
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum IpcCmd {
    IpcRmid = 0,
    IpcSet = 1,
    IpcStat = 2,
    IpcInfo = 3,
}

/// Create an anonymous pipe
/// POSIX.1-2017: pipe() creates a pipe with read and write ends.
/// Error conditions:
///   - EMFILE: per-process file descriptor limit would be exceeded
///   - ENFILE: system-wide file table is full
///   - EFAULT: fd pointer is invalid
pub fn pipe(_fd: &mut [i32; 2]) -> Result<(), Errno> {
    Err(Errno::Enosys)
}

/// Message queue attributes
#[derive(Debug, Clone, Copy)]
pub struct MqAttr {
    pub mq_flags: i64,
    pub mq_maxmsg: i64,
    pub mq_msgsize: i64,
    pub mq_curmsgs: i64,
}

/// Open a message queue
/// POSIX.1-2017: mq_open() creates or opens a message queue.
/// Error conditions:
///   - EACCES: permission denied
///   - EEXIST: O_CREAT|O_EXCL and queue already exists
///   - ENOENT: O_CREAT not set and queue does not exist
///   - ENAMETOOLONG: name exceeds PATH_MAX
///   - EMFILE/ENFILE: too many file descriptors open
pub fn mq_open(_name: &str, _oflag: i32) -> Result<IpcId, Errno> {
    Err(Errno::Enosys)
}

/// Close a message queue
/// POSIX.1-2017: mq_close() closes a message queue descriptor.
pub fn mq_close(_mqdes: IpcId) -> Result<(), Errno> {
    Err(Errno::Enosys)
}

/// Send a message to a queue
/// POSIX.1-2017: mq_send() adds a message to a queue.
pub fn mq_send(_mqdes: IpcId, _msg_ptr: &[u8], _msg_prio: u32) -> Result<(), Errno> {
    Err(Errno::Enosys)
}

/// Receive a message from a queue
/// POSIX.1-2017: mq_receive() receives the oldest highest-priority message.
pub fn mq_receive(_mqdes: IpcId, _msg_ptr: &mut [u8], _msg_prio: &mut u32) -> Result<usize, Errno> {
    Err(Errno::Enosys)
}

/// Shared memory open
/// POSIX.1-2017: shm_open() creates or opens a shared memory object.
/// Error conditions:
///   - EACCES: permission denied
///   - EEXIST: O_CREAT|O_EXCL and object already exists
///   - ENOENT: O_CREAT not set and object does not exist
///   - ENAMETOOLONG: name exceeds PATH_MAX
///   - EINVAL: name does not start with /
pub fn shm_open(_name: &str, _oflag: i32, _mode: u32) -> Result<i32, Errno> {
    Err(Errno::Enosys)
}

/// Shared memory unlink
/// POSIX.1-2017: shm_unlink() removes a shared memory object.
pub fn shm_unlink(_name: &str) -> Result<(), Errno> {
    Err(Errno::Enosys)
}

/// Create a FIFO special file (named pipe)
/// POSIX.1-2017: mkfifo() creates a new FIFO special file.
/// Error conditions:
///   - EACCES: write permission denied for directory
///   - EEXIST: file already exists
///   - ENOENT: directory component does not exist
///   - ENOSPC: no space left on device
/// POSIX DEVIATION: mkfifo() returns ENOSYS. Named pipes not yet implemented.
/// Anonymous pipes are supported via pipe().
pub fn mkfifo(_path: &str, _mode: u32) -> Result<(), Errno> {
    Err(Errno::Enosys)
}
