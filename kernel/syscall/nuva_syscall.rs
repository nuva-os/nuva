/*
 * Nuva OS - Kernel - Syscall - NuvaSyscall
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
 * Nuva OS - Kernel - Nuva Native System Call Interface
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Nuva native system call number space and dispatch.
 * Independent from POSIX system call numbering (0x0001_0000 - 0x0001_FFFF).
 * Nuva native calls occupy 0x0000_0000 - 0x0000_FFFF.
 */

use crate::types::{NuvaProcessId, NuvaError, NuvaCapabilityId, NuvaAccessRight};

// ============================================================================
// Nuva Native System Call Numbers (0x0000_0000 - 0x0000_FFFF)
// ============================================================================

/// Nuva system call number base
pub const NUVA_SYSCALL_BASE: u32 = 0x0000_0000;

// Process management syscalls (0x01 - 0x0F)
pub const NUVA_PROCESS_CREATE:    u32 = NUVA_SYSCALL_BASE | 0x01;
pub const NUVA_PROCESS_EXECUTE:   u32 = NUVA_SYSCALL_BASE | 0x02;
pub const NUVA_PROCESS_TERMINATE: u32 = NUVA_SYSCALL_BASE | 0x03;
pub const NUVA_PROCESS_YIELD:     u32 = NUVA_SYSCALL_BASE | 0x04;

// Memory management syscalls (0x10 - 0x1F)
pub const NUVA_MEMORY_ALLOCATE:   u32 = NUVA_SYSCALL_BASE | 0x10;
pub const NUVA_MEMORY_DEALLOCATE: u32 = NUVA_SYSCALL_BASE | 0x11;
pub const NUVA_MEMORY_PROTECT:    u32 = NUVA_SYSCALL_BASE | 0x12;
pub const NUVA_MEMORY_MAP:        u32 = NUVA_SYSCALL_BASE | 0x13;

// IPC syscalls (0x20 - 0x2F)
pub const NUVA_IPC_PORT_CREATE:   u32 = NUVA_SYSCALL_BASE | 0x20;
pub const NUVA_IPC_PORT_DESTROY:  u32 = NUVA_SYSCALL_BASE | 0x21;
pub const NUVA_IPC_SEND:          u32 = NUVA_SYSCALL_BASE | 0x22;
pub const NUVA_IPC_RECEIVE:       u32 = NUVA_SYSCALL_BASE | 0x23;
pub const NUVA_IPC_CALL:          u32 = NUVA_SYSCALL_BASE | 0x24;
pub const NUVA_IPC_REPLY:         u32 = NUVA_SYSCALL_BASE | 0x25;
pub const NUVA_IPC_FORWARD:       u32 = NUVA_SYSCALL_BASE | 0x26;

// File operation syscalls (0x30 - 0x3F)
pub const NUVA_FILE_OPEN:         u32 = NUVA_SYSCALL_BASE | 0x30;
pub const NUVA_FILE_CLOSE:        u32 = NUVA_SYSCALL_BASE | 0x31;
pub const NUVA_FILE_READ:         u32 = NUVA_SYSCALL_BASE | 0x32;
pub const NUVA_FILE_WRITE:        u32 = NUVA_SYSCALL_BASE | 0x33;
pub const NUVA_FILE_SEEK:         u32 = NUVA_SYSCALL_BASE | 0x34;
pub const NUVA_FILE_IOCTL:        u32 = NUVA_SYSCALL_BASE | 0x35;

// Capability operation syscalls (0x40 - 0x4F)
pub const NUVA_CAPABILITY_GRANT:  u32 = NUVA_SYSCALL_BASE | 0x40;
pub const NUVA_CAPABILITY_REVOKE: u32 = NUVA_SYSCALL_BASE | 0x41;
pub const NUVA_CAPABILITY_CHECK:  u32 = NUVA_SYSCALL_BASE | 0x42;
pub const NUVA_CAPABILITY_TRANSFER: u32 = NUVA_SYSCALL_BASE | 0x43;

// Event notification syscalls (0x50 - 0x5F)
pub const NUVA_EVENT_REGISTER:    u32 = NUVA_SYSCALL_BASE | 0x50;
pub const NUVA_EVENT_NOTIFY:      u32 = NUVA_SYSCALL_BASE | 0x51;
pub const NUVA_EVENT_WAIT:        u32 = NUVA_SYSCALL_BASE | 0x52;

// Diagnostic syscalls (0x60 - 0x6F)
pub const NUVA_DIAG_QUERY:        u32 = NUVA_SYSCALL_BASE | 0x60;
pub const NUVA_DIAG_STATS:        u32 = NUVA_SYSCALL_BASE | 0x61;

/// POSIX system call number base (when POSIX feature is enabled)
#[cfg(feature = "posix")]
pub const POSIX_SYSCALL_BASE: u32 = 0x0001_0000;

/// Dispatch a Nuva native system call.
///
/// This function handles system calls in the Nuva native number space
/// (0x0000_0000 - 0x0000_FFFF). POSIX system calls are handled
/// separately by posix_syscall_dispatch when the POSIX feature is enabled.
pub fn nuva_syscall_dispatch(call_num: u32, args: &[u64]) -> Result<u64, NuvaError> {
    match call_num {
        NUVA_PROCESS_CREATE => nuva_process_create(args),
        NUVA_PROCESS_EXECUTE => nuva_process_execute(args),
        NUVA_PROCESS_TERMINATE => nuva_process_terminate(args),
        NUVA_PROCESS_YIELD => nuva_process_yield(args),

        NUVA_MEMORY_ALLOCATE => nuva_memory_allocate(args),
        NUVA_MEMORY_DEALLOCATE => nuva_memory_deallocate(args),
        NUVA_MEMORY_PROTECT => nuva_memory_protect(args),
        NUVA_MEMORY_MAP => nuva_memory_map(args),

        NUVA_IPC_PORT_CREATE => nuva_ipc_port_create(args),
        NUVA_IPC_PORT_DESTROY => nuva_ipc_port_destroy(args),
        NUVA_IPC_SEND => nuva_ipc_send(args),
        NUVA_IPC_RECEIVE => nuva_ipc_receive(args),
        NUVA_IPC_CALL => nuva_ipc_call(args),
        NUVA_IPC_REPLY => nuva_ipc_reply(args),
        NUVA_IPC_FORWARD => nuva_ipc_forward(args),

        NUVA_FILE_OPEN => nuva_file_open(args),
        NUVA_FILE_CLOSE => nuva_file_close(args),
        NUVA_FILE_READ => nuva_file_read(args),
        NUVA_FILE_WRITE => nuva_file_write(args),
        NUVA_FILE_SEEK => nuva_file_seek(args),
        NUVA_FILE_IOCTL => nuva_file_ioctl(args),

        NUVA_CAPABILITY_GRANT => nuva_capability_grant(args),
        NUVA_CAPABILITY_REVOKE => nuva_capability_revoke(args),
        NUVA_CAPABILITY_CHECK => nuva_capability_check(args),
        NUVA_CAPABILITY_TRANSFER => nuva_capability_transfer(args),

        NUVA_EVENT_REGISTER => nuva_event_register(args),
        NUVA_EVENT_NOTIFY => nuva_event_notify(args),
        NUVA_EVENT_WAIT => nuva_event_wait(args),

        NUVA_DIAG_QUERY => nuva_diag_query(args),
        NUVA_DIAG_STATS => nuva_diag_stats(args),

        _ => Err(NuvaError::InvalidCall),
    }
}

/// Unified system call dispatcher.
/// Routes to Nuva native, POSIX, or Vulkan dispatch based on call number.
pub fn syscall_dispatch(call_num: u32, args: &[u64]) -> Result<u64, NuvaError> {
    if call_num < 0x0001_0000 && call_num < 0x0070 {
        nuva_syscall_dispatch(call_num, args)
    } else if call_num >= 0x0070 && call_num <= 0x008F {
        // Vulkan system call space (0x70-0x8F)
        #[cfg(feature = "vulkan")]
        {
            crate::syscall::nv_vulkan_syscall::nv_vulkan_syscall_dispatch(call_num, args)
        }
        #[cfg(not(feature = "vulkan"))]
        {
            let _ = args;
            Err(NuvaError::InvalidCall)
        }
    } else if call_num >= 0x0001_0000 {
        #[cfg(feature = "posix")]
        {
            posix_syscall_dispatch(call_num, args)
        }
        #[cfg(not(feature = "posix"))]
        {
            let _ = args;
            Err(NuvaError::InvalidCall)
        }
    } else {
        let _ = args;
        Err(NuvaError::InvalidCall)
    }
}

/// POSIX system call dispatch (only available with POSIX feature)
#[cfg(feature = "posix")]
fn posix_syscall_dispatch(_call_num: u32, _args: &[u64]) -> Result<u64, NuvaError> {
    // POSIX syscall dispatch implementation
    // Adapts POSIX calls to Nuva native interfaces
    Err(NuvaError::InvalidCall)
}

// ============================================================================
// Nuva Native System Call Implementations (stubs for architecture)
// All implementations require NuvaCapability token verification.
// ============================================================================

fn check_capability(cap_id: u64) -> Result<NuvaCapabilityId, NuvaError> {
    if cap_id == 0 {
        return Err(NuvaError::CapabilityDenied);
    }
    Ok(NuvaCapabilityId::new(cap_id))
}

fn nuva_process_create(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    // TODO: Implement with process manager
    Ok(NuvaProcessId::new(0).as_u64())
}

fn nuva_process_execute(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_process_terminate(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_process_yield(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_memory_allocate(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_memory_deallocate(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_memory_protect(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_memory_map(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_ipc_port_create(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_ipc_port_destroy(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_ipc_send(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_ipc_receive(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_ipc_call(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_ipc_reply(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_ipc_forward(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_file_open(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_file_close(args: &[u64]) -> Result<u64, NuvaError> {
    let _ = args;
    Ok(0)
}

fn nuva_file_read(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(1).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_file_write(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(1).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_file_seek(args: &[u64]) -> Result<u64, NuvaError> {
    let _ = args;
    Ok(0)
}

fn nuva_file_ioctl(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(1).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_capability_grant(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_capability_revoke(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_capability_check(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_capability_transfer(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_event_register(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_event_notify(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_event_wait(args: &[u64]) -> Result<u64, NuvaError> {
    let cap = check_capability(*args.get(0).unwrap_or(&0))?;
    let _ = cap;
    Ok(0)
}

fn nuva_diag_query(args: &[u64]) -> Result<u64, NuvaError> {
    let _ = args;
    Ok(0)
}

fn nuva_diag_stats(args: &[u64]) -> Result<u64, NuvaError> {
    let _ = args;
    Ok(0)
}
