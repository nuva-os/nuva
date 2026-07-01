/*
 * Nuva OS - Kernel - PowerMgmt - Nvpowermgr - Api
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
 * Nuva OS - Kernel - NvPowerMgr System API
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * NvPowerMgr system call interface.
 * All interfaces require NuvaCapabilityId as first parameter.
 */

use crate::kernel::error::KernelResult;
use crate::kernel::power_mgmt::nvpowermgr::{NvPowerMgr, get_nv_powermgr};
use crate::kernel::power_mgmt::nvpowermgr::stats::get_nv_powermgr_stats;
use crate::kernel::error::KernelError;

/// NuvaCapabilityId type
pub type NuvaCapabilityId = u64;

/// Set power budget
///
/// @param cap: Caller capability token
/// @param budget_mw: Power budget in milliwatts
/// @return: Ok on success
pub fn nv_power_set_budget(cap: NuvaCapabilityId, budget_mw: u32) -> KernelResult<()> {
    let _ = cap;
    let mgr = get_nv_powermgr();
    if !mgr.is_initialized() {
        return Err(crate::kernel::error::KernelError::InvalidArgument);
    }
    // TODO: Delegate to PowerBudgetManager
    let _ = budget_mw;
    Ok(())
}

/// Get current power consumption
///
/// @param cap: Caller capability token
/// @return: (total_mw, budget_mw) on success
pub fn nv_power_get_consumption(cap: NuvaCapabilityId) -> KernelResult<(u32, u32)> {
    let _ = cap;
    Ok((0, 0))
}

/// Get green computing metrics
///
/// @param cap: Caller capability token
/// @return: (pue_x100, carbon_g, efficiency_score) on success
pub fn nv_power_get_green_metrics(cap: NuvaCapabilityId) -> KernelResult<(u32, u64, u32)> {
    let _ = cap;
    Ok((100, 0, 0))
}

/// Set per-device DVFS level
///
/// @param cap: Caller capability token
/// @param device_index: Target device
/// @param level: DVFS level
/// @return: Ok on success
pub fn nv_power_set_device_dvfs(cap: NuvaCapabilityId, device_index: usize, level: u16) -> KernelResult<()> {
    let _ = cap;
    let _ = device_index;
    let _ = level;
    Ok(())
}

/// Set per-device power state
///
/// @param cap: Caller capability token
/// @param device_index: Target device
/// @param sleep_level: Target sleep level (0-3)
/// @return: Ok on success
pub fn nv_power_set_device_state(cap: NuvaCapabilityId, device_index: usize, sleep_level: u8) -> KernelResult<()> {
    let _ = cap;
    let _ = device_index;
    let _ = sleep_level;
    Ok(())
}

/// Get per-device thermal status
///
/// @param cap: Caller capability token
/// @param device_index: Target device
/// @return: (temp_c, is_throttled, sensor_state) on success
pub fn nv_power_get_thermal(cap: NuvaCapabilityId, device_index: usize) -> KernelResult<(u32, bool, u8)> {
    let _ = cap;
    let _ = device_index;
    Ok((25, false, 0))
}

/// Evaluate power impact of a scheduling decision
///
/// @param cap: Caller capability token
/// @param decision_id: Scheduling decision ID
/// @return: (estimated_power_mw, efficiency_score) on success
pub fn nv_power_evaluate_impact(cap: NuvaCapabilityId, decision_id: u64) -> KernelResult<(u32, u32)> {
    let _ = cap;
    let _ = decision_id;
    Ok((0, 50))
}

/// Get power management statistics
///
/// @param cap: Caller capability token
/// @return: (cycles, dvfs_adjustments, energy_saved_mwh, budget_violations)
pub fn nv_power_get_stats(cap: NuvaCapabilityId) -> KernelResult<(u64, u64, u64, u64)> {
    let _ = cap;
    let stats = get_nv_powermgr_stats();
    Ok((
        stats.optimization_cycles.load(core::sync::atomic::Ordering::Acquire),
        stats.dvfs_adjustments.load(core::sync::atomic::Ordering::Acquire),
        stats.total_energy_saved_mwh.load(core::sync::atomic::Ordering::Acquire),
        stats.budget_violations.load(core::sync::atomic::Ordering::Acquire),
    ))
}