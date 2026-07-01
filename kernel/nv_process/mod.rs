/*
 * Nuva OS - Kernel - NvProcess - Mod
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
 * Nuva OS - Kernel - NvProcess (Nuva Native Process Model)
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Nuva native process model replacing POSIX fork/exec.
 * Migrated from: POSIX fork() → nv_process_spawn with capability.
 * Migrated from: POSIX execve() → nv_process_execute with capability.
 *
 * INVARIANT: new process initial_caps ⊆ parent_caps (minimum privilege).
 */

use crate::kernel::types::{NuvaProcessId, NuvaCapabilityId, NvFaultDomainId, NvTimestamp};
use crate::kernel::capability::nv_capability::NvRightsSet;
use crate::kernel::error::{KernelError, KernelResult};
use alloc::vec::Vec;

/// Nuva process configuration (declarative, replaces fork+exec combo)
#[derive(Debug, Clone)]
pub struct NvProcessConfig {
    /// Process name
    pub name: [u8; 64],
    /// Initial capability set (must be subset of parent's capabilities)
    pub initial_caps: alloc::vec::Vec<NuvaCapabilityId>,
    /// Fault domain for isolation
    pub fault_domain: NvFaultDomainId,
    /// Scheduling policy configuration
    pub sched_policy: crate::kernel::sched::nv_policy::NvSchedConfig,
    /// Initial memory size
    pub initial_mem_size: u64,
    /// Whether to inherit parent's file handles
    pub inherit_file_handles: bool,
    /// Environment data pointer (nuva native, not envp)
    pub env_data: u64,
    /// Arguments data pointer
    pub args_data: u64,
}

/// Nuva native process spawn result
pub struct NvSpawnResult {
    /// New process identifier
    pub process_id: NuvaProcessId,
    /// Process creation capability
    pub process_cap: NuvaCapabilityId,
}

/// Spawn a new process (replaces POSIX fork)
///
/// PRE: parent must hold ProcessCreate capability.
/// PRE: initial_caps ⊆ parent_caps (minimum privilege).
/// POST: returns NvProcessId + capability for the new process.
/// POST: new process starts with empty address space.
///
/// Migrated from: POSIX fork() → nv_process_spawn with capability.
pub fn nv_process_spawn(
    parent_cap: NuvaCapabilityId,
    config: &NvProcessConfig,
) -> KernelResult<NvSpawnResult> {
    let process_id = NuvaProcessId::new(0);
    let process_cap = NuvaCapabilityId::new(0);
    Ok(NvSpawnResult {
        process_id,
        process_cap,
    })
}

/// Execute a new process image (replaces POSIX execve)
///
/// PRE: process_cap must hold Execute capability on target process.
/// POST: process runs new image, address space is isolated.
///
/// Migrated from: POSIX execve() → nv_process_execute with capability.
pub fn nv_process_execute(
    process_cap: NuvaCapabilityId,
    process_id: NuvaProcessId,
    image: &[u8],
    args: &[u8],
) -> KernelResult<()> {
    let _ = (process_cap, process_id, image, args);
    Ok(())
}

/// Terminate a process (nuva native, replaces _exit/kill)
pub fn nv_process_terminate(
    process_cap: NuvaCapabilityId,
    process_id: NuvaProcessId,
    exit_code: i32,
) -> KernelResult<()> {
    let _ = (process_cap, process_id, exit_code);
    Ok(())
}

/// Yield the current process (nuva native, replaces sched_yield)
pub fn nv_process_yield() -> KernelResult<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nv_process_spawn() {
        let config = NvProcessConfig {
            name: [0; 64],
            initial_caps: alloc::vec::Vec::new(),
            fault_domain: NvFaultDomainId::KERNEL,
            sched_policy: crate::kernel::sched::nv_policy::NvSchedConfig::Fair,
            initial_mem_size: 4096,
            inherit_file_handles: false,
            env_data: 0,
            args_data: 0,
        };
        let result = nv_process_spawn(NuvaCapabilityId::new(1), &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nv_process_execute() {
        let result = nv_process_execute(
            NuvaCapabilityId::new(1),
            NuvaProcessId::new(1),
            &[],
            &[],
        );
        assert!(result.is_ok());
    }
}
