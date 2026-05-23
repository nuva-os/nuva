/*
 * Nuva OS - Syslib - POSIX unistd.h compatibility
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

use super::errno::Errno;

/// Process identity information
pub struct ProcessIdentity {
    pub pid: i32,
    pub ppid: i32,
    pub uid: u32,
    pub gid: u32,
    pub euid: u32,
    pub egid: u32,
}

/// Create a copy of the current process
/// POSIX.1-2017: fork() creates a new process by duplicating the calling process.
/// Returns: 0 in child, child PID in parent, or error.
/// Error conditions:
///   - ENOMEM: Insufficient kernel resources to create a new process
///   - EAGAIN: System limit on total number of processes would be exceeded
/// POSIX DEVIATION: fork() returns ENOSYS. Microkernel does not support full
/// address space duplication. Use spawn() for process creation.
pub fn fork() -> Result<i32, Errno> {
    Err(Errno::Enosys)
}

/// Replace the current process image
/// POSIX.1-2017: execve() replaces the current process with a new program.
/// Error conditions:
///   - ENOENT: path does not exist
///   - EACCES: execute permission denied for path
///   - ENOEXEC: file is not in executable format
///   - ENOMEM: insufficient memory for new process image
/// POSIX DEVIATION: execve() returns ENOSYS. Microkernel uses capability-based
/// process loading. Use process_spawn_with_capabilities() instead.
pub fn execve(_path: &str, _argv: &[&str], _envp: &[&str]) -> Result<(), Errno> {
    Err(Errno::Enosys)
}

/// Wait for a child process to change state
/// POSIX.1-2017: waitpid() suspends execution until a child changes state.
/// Error conditions:
///   - ECHILD: no child process matching pid
///   - EINTR: interrupted by signal (if SA_RESTART not set)
///   - EINVAL: invalid options
pub fn waitpid(_pid: i32, _status: &mut i32, _options: i32) -> Result<i32, Errno> {
    Err(Errno::Enosys)
}

/// Terminate the calling process
/// POSIX.1-2017: _exit() terminates the process immediately.
/// This function never returns.
pub fn exit(_status: i32) -> ! {
    loop {}
}

/// Get the process ID
/// POSIX.1-2017: getpid() returns the process ID of the calling process.
pub fn getpid() -> i32 {
    0
}

/// Get the parent process ID
/// POSIX.1-2017: getppid() returns the parent process ID.
pub fn getppid() -> i32 {
    0
}

/// Get the real user ID
/// POSIX.1-2017: getuid() returns the real user ID.
pub fn getuid() -> u32 {
    0
}

/// Get the effective user ID
/// POSIX.1-2017: geteuid() returns the effective user ID.
pub fn geteuid() -> u32 {
    0
}

/// Get the real group ID
/// POSIX.1-2017: getgid() returns the real group ID.
pub fn getgid() -> u32 {
    0
}

/// Get the effective group ID
/// POSIX.1-2017: getegid() returns the effective group ID.
pub fn getegid() -> u32 {
    0
}

/// Set the user ID
/// POSIX.1-2017: setuid() sets the real, effective, and saved-set user IDs.
pub fn setuid(_uid: u32) -> Result<(), Errno> {
    Err(Errno::Enosys)
}

/// Set the group ID
/// POSIX.1-2017: setgid() sets the real, effective, and saved-set group IDs.
pub fn setgid(_gid: u32) -> Result<(), Errno> {
    Err(Errno::Enosys)
}

/// Get process identity as a batch operation
pub fn get_process_identity() -> ProcessIdentity {
    ProcessIdentity {
        pid: getpid(),
        ppid: getppid(),
        uid: getuid(),
        gid: getgid(),
        euid: geteuid(),
        egid: getegid(),
    }
}

// ============================================================================
// POSIX Thread (pthread) Stubs - DEVIATION
// ============================================================================

/// POSIX DEVIATION: pthread_create() returns ENOSYS. Microkernel uses L4-style
/// lightweight tasks with different semantics. Use task_spawn() instead.
pub fn pthread_create() -> Result<(), Errno> {
    Err(Errno::Enosys)
}

/// POSIX DEVIATION: pthread_join() returns ENOSYS. See pthread_create deviation.
pub fn pthread_join() -> Result<(), Errno> {
    Err(Errno::Enosys)
}

/// POSIX DEVIATION: pthread_mutex_lock() returns ENOSYS. Use kernel sync primitives.
pub fn pthread_mutex_lock() -> Result<(), Errno> {
    Err(Errno::Enosys)
}
