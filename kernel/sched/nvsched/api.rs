/*
 * Nuva OS - Kernel - Sched - Nvsched - Api
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
 * Nuva OS - Kernel - NvScheduler System API
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * NvScheduler system call interface.
 * All interfaces require NuvaCapabilityId as first parameter.
 */

use crate::kernel::error::KernelResult;
use crate::kernel::sched::nvsched::{
    NvScheduler, NvSchedMode, get_nv_scheduler,
};
use crate::kernel::sched::nvsched::stats::get_nv_sched_stats;

/// NuvaCapabilityId type (placeholder for capability system)
pub type NuvaCapabilityId = u64;

/// Submit a task for AI-driven scheduling
///
/// @param cap: Caller capability token
/// @param pid: Process ID to schedule
/// @param sched_class: Desired scheduling class (0-3)
/// @return: Decision ID on success
pub fn nv_sched_submit_task(cap: NuvaCapabilityId, pid: u32, sched_class: u8) -> KernelResult<u64> {
    let _ = cap;
    let scheduler = get_nv_scheduler();

    if !scheduler.is_initialized() {
        return Err(crate::kernel::error::KernelError::InvalidArgument);
    }

    let _ = sched_class;
    let _ = pid;
    Ok(0)
}

/// Set scheduling policy configuration
///
/// @param cap: Caller capability token
/// @param mode: Scheduling mode (0=AI, 1=DeclPolicy, 2=Traditional)
/// @return: Ok on success
pub fn nv_sched_set_policy(cap: NuvaCapabilityId, mode: u8) -> KernelResult<()> {
    let _ = cap;
    let scheduler = get_nv_scheduler();
    scheduler.set_mode(NvSchedMode::from_u8(mode))
}

/// Get current scheduling decision for a process
///
/// @param cap: Caller capability token
/// @param pid: Process ID
/// @return: (decision_id, target_device, confidence) on success
pub fn nv_sched_get_decision(cap: NuvaCapabilityId, pid: u32) -> KernelResult<(u64, u8, u8)> {
    let _ = cap;
    let _ = pid;
    Ok((0, 0, 0))
}

/// Set scheduling mode
///
/// @param cap: Caller capability token
/// @param mode: Scheduling mode
/// @return: Ok on success
pub fn nv_sched_set_mode(cap: NuvaCapabilityId, mode: NvSchedMode) -> KernelResult<()> {
    let _ = cap;
    let scheduler = get_nv_scheduler();
    scheduler.set_mode(mode)
}

/// Get scheduling statistics
///
/// @param cap: Caller capability token
/// @return: (ai_decisions, fallback_decisions, ai_ratio_pct, avg_latency_us)
pub fn nv_sched_get_stats(cap: NuvaCapabilityId) -> KernelResult<(u64, u64, u32, u32)> {
    let _ = cap;
    let stats = get_nv_sched_stats();
    Ok((
        stats.ai_decisions.load(core::sync::atomic::Ordering::Acquire),
        stats.fallback_decisions.load(core::sync::atomic::Ordering::Acquire),
        stats.ai_decision_ratio_pct(),
        stats.avg_inference_latency_us(),
    ))
}